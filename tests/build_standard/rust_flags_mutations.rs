//! Source-mutation contracts for the copied production Makefile.
//!
//! Each contract applies one uniquely located edit to a scratch copy of the
//! candidate Makefile and checks that the baseline route detects exactly that
//! difference.

use rstest::rstest;

use super::{
    super::{
        make::Host,
        make_input_env::{
            self as make_input,
            Invocation,
            NEXTEST_OPTIONS,
            Observation,
            RunRequest,
            RustFlagsInput,
        },
    },
    cases::{FailureCase, Mutation, expected_calls, failure_contract, reached_failure},
    exact_mutation_delta,
    mutation_diagnostic,
    require_calls,
};

/// Proves the source-copy contract rejects representative Makefile mutations.
#[rstest]
#[case::mutation_unexport(Mutation::Unexport)]
#[case::mutation_nextest_option(Mutation::NextestOption)]
#[case::mutation_fallback_leak(Mutation::FallbackLeak)]
#[case::mutation_restatement(Mutation::Restatement)]
#[case::mutation_failure_mask(Mutation::FailureMask)]
fn make_test_contract_detects_source_mutations(#[case] mutation: Mutation) -> Result<(), String> {
    make_input::with_scratch(|scratch| check_source_mutation(scratch, mutation))
}

/// Runs one source mutation only after its unmodified production route passes.
fn check_source_mutation(scratch: &make_input::Scratch, mutation: Mutation) -> Result<(), String> {
    let baseline = mutation_request(mutation, None);
    make_input::check_target(scratch, &baseline)?;
    let expected = expected_calls(&baseline)?;
    if mutation == Mutation::FailureMask {
        check_failure_mask_baseline(scratch)?;
    }
    let changed = mutated_makefile(mutation, &scratch.read_makefile()?)?;
    scratch.write_makefile(&changed)?;
    let failure = if mutation == Mutation::FailureMask {
        Some(FailureCase::NextestRun)
    } else {
        None
    };
    let observation = scratch.run(&mutation_request(mutation, failure))?;
    check_mutated_route(mutation, &expected, &observation)
}

/// Selects the literal Make route and Nextest branch for one finite mutation name.
fn mutation_request(mutation: Mutation, failure: Option<FailureCase>) -> RunRequest<'static> {
    let (target, nextest_available) = match mutation {
        Mutation::Unexport | Mutation::NextestOption => ("test-ci", true),
        Mutation::FallbackLeak => ("test", false),
        Mutation::Restatement => ("typecheck", true),
        Mutation::FailureMask => ("test", true),
    };
    RunRequest {
        target,
        host: Host::Linux,
        rust_flags: RustFlagsInput::CommandLine,
        nextest_available,
        nextest_flags: NEXTEST_OPTIONS,
        failure: failure.map_or("", FailureCase::as_str),
    }
}

/// Applies exactly one uniquely located mutation to the copied candidate Makefile.
fn mutated_makefile(mutation: Mutation, original: &str) -> Result<String, String> {
    match mutation {
        Mutation::Unexport => replace_once(original, "unexport RUST_FLAGS\n", ""),
        Mutation::NextestOption => replace_once(
            original,
            "nextest run --profile ci $(TEST_FLAGS) $(NEXTEST_FLAGS) $(BUILD_JOBS)",
            "nextest run --profile ci $(TEST_FLAGS) $(BUILD_JOBS)",
        ),
        Mutation::FallbackLeak => replace_once(
            original,
            "$(CARGO) test $(TEST_FLAGS) $(BUILD_JOBS); \\",
            "$(CARGO) test $(TEST_FLAGS) $(NEXTEST_FLAGS) $(BUILD_JOBS); \\",
        ),
        Mutation::Restatement => replace_once(
            original,
            "$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)\" $(CARGO) check",
            "$(STANDARD_RUSTFLAGS)\" $(CARGO) check",
        ),
        Mutation::FailureMask => replace_once(
            original,
            concat!(
                "$(CARGO) nextest run --profile default $(TEST_FLAGS) ",
                "$(NEXTEST_FLAGS) $(BUILD_JOBS); \\"
            ),
            concat!(
                "$(CARGO) nextest run --profile default $(TEST_FLAGS) ",
                "$(NEXTEST_FLAGS) $(BUILD_JOBS) || true; \\"
            ),
        ),
    }
}

/// Confirms the original recipe propagates the named failing Nextest leaf.
fn check_failure_mask_baseline(scratch: &make_input::Scratch) -> Result<(), String> {
    let request = mutation_request(Mutation::FailureMask, Some(FailureCase::NextestRun));
    let observation = scratch.run(&request)?;
    failure_contract(&observation, FailureCase::NextestRun)
}

/// Rejects an unsuccessful mutated route before checking its one intended contract difference.
fn check_mutated_route(
    mutation: Mutation,
    expected: &[Invocation],
    observation: &Observation,
) -> Result<(), String> {
    if !observation.status.success() {
        return Err(format!("{mutation} mutation made its route fail"));
    }
    if mutation == Mutation::FailureMask {
        check_failure_mask_mutation(expected, observation)
    } else {
        check_ordinary_mutation(mutation, expected, observation)
    }
}

/// Requires the swallowed-error mutation to reach its failing leaf and report only lost
/// propagation.
fn check_failure_mask_mutation(
    expected: &[Invocation],
    observation: &Observation,
) -> Result<(), String> {
    require_calls(expected, &observation.calls, "failure-mask route")?;
    if !reached_failure(observation, FailureCase::NextestRun) {
        return Err("failure-mask mutation missed the failing Nextest leaf".to_owned());
    }
    match failure_contract(observation, FailureCase::NextestRun) {
        Err(error) if error == "make did not propagate the controlled nextest-run failure" => {
            Ok(())
        }
        Err(error) => Err(format!(
            "failure-mask produced an unrelated rejection: {error}"
        )),
        Ok(()) => Err("failure-mask was not rejected by the failure contract".to_owned()),
    }
}

/// Requires only the exact baseline-row mismatch assigned to a non-failure mutation.
fn check_ordinary_mutation(
    mutation: Mutation,
    expected: &[Invocation],
    observation: &Observation,
) -> Result<(), String> {
    let diagnostic = mutation_diagnostic(mutation, expected, &observation.calls)?;
    match require_calls(expected, &observation.calls, &mutation.to_string()) {
        Err(error) if error == diagnostic => {
            exact_mutation_delta(mutation, expected, &observation.calls)
        }
        Err(error) => Err(format!("{mutation} caused an unrelated rejection: {error}")),
        Ok(()) => Err(format!(
            "{mutation} was not detected by the baseline contract"
        )),
    }
}

/// Replaces one unique source fragment in the copied Makefile.
fn replace_once(source: &str, from: &str, to: &str) -> Result<String, String> {
    let count = source.matches(from).count();
    if count != 1 {
        return Err(format!(
            "expected one mutation match, found {count}: {from}"
        ));
    }
    Ok(source.replacen(from, to, 1))
}
