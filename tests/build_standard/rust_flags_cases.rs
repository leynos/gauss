//! Exercises Rust flag and Nextest routing through the copied production Makefile.

use rstest::rstest;

use super::super::make::Host;
use super::super::make_input_env::{
    self as make_input, Invocation, NEXTEST_OPTIONS, Observation, RunRequest, RustFlagsInput,
};

/// Pure expected-row queries for supported Make routes.
#[path = "rust_flags_expected.rs"]
mod expected;

/// Re-exports the expected-row query through the established cases module.
pub(crate) use expected::expected_calls;

/// Requires Make to fail after the named fake leaf command returns non-zero.
fn failure_contract(observation: &Observation, failure: &str) -> Result<(), String> {
    if !observation.status.success() && reached_failure(observation, failure) {
        Ok(())
    } else {
        Err(format!(
            "make did not propagate the controlled {failure} failure"
        ))
    }
}

/// Reports whether the named fake failing leaf was recorded.
fn reached_failure(observation: &Observation, failure: &str) -> bool {
    observation.calls.iter().any(|call| match failure {
        "nextest-run" => {
            call.arguments.first().map(String::as_str) == Some("nextest")
                && call.arguments.get(1).map(String::as_str) == Some("run")
        }
        "cargo-test" => {
            call.arguments.first().map(String::as_str) == Some("test")
                && !call.arguments.iter().any(|argument| argument == "--doc")
        }
        "doctest" => {
            call.arguments.first().map(String::as_str) == Some("test")
                && call.arguments.iter().any(|argument| argument == "--doc")
        }
        _ => false,
    })
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

/// Names the row mutation used to exercise each compiler environment field.
type CompilerFlagMutator = fn(&mut Invocation, &str);

/// Checks compiler-flag whitespace equivalence without relaxing other recorded fields.
#[test]
fn compiler_flag_comparison_ignores_only_ascii_whitespace() {
    const FLAGS: &str = "--cfg alpha -D warnings --opaque=original";
    const SPACED_FLAGS: &str = " \t--cfg   alpha \n-D warnings\t--opaque=original  ";
    const INVALID_FLAGS: [(&str, &str); 4] = [
        ("missing flag", "--cfg alpha -D warnings"),
        (
            "reordered flags",
            "-D warnings --cfg alpha --opaque=original",
        ),
        (
            "duplicate flag",
            "--cfg alpha -D warnings --opaque=original --cfg alpha",
        ),
        (
            "changed unknown token",
            "--cfg alpha -D warnings --opaque=changed",
        ),
    ];
    let baseline = Invocation {
        arguments: vec!["cargo test".to_owned()],
        rust_flags_present: false,
        rustflags: FLAGS.to_owned(),
        rustdocflags: FLAGS.to_owned(),
    };
    let compare = |actual: &Invocation| {
        super::require_calls(
            std::slice::from_ref(&baseline),
            std::slice::from_ref(actual),
            "oracle",
        )
    };
    let fields: [(&str, CompilerFlagMutator); 2] = [
        ("RUSTFLAGS", |row, flags| row.rustflags = flags.to_owned()),
        ("RUSTDOCFLAGS", |row, flags| {
            row.rustdocflags = flags.to_owned();
        }),
    ];
    for (field, set_flags) in fields {
        let mut spaced = baseline.clone();
        set_flags(&mut spaced, SPACED_FLAGS);
        compare(&spaced).expect("ASCII whitespace changes should preserve flag tokens");
        for &(change, flags) in &INVALID_FLAGS {
            let mut observed = baseline.clone();
            set_flags(&mut observed, flags);
            let error = compare(&observed).expect_err("token changes must remain visible");
            assert!(
                error.starts_with(&format!("oracle {field} mismatch at invocation 1")),
                "{change} should fail on {field}: {error}"
            );
        }
    }
    let mut changed_argument = baseline.clone();
    changed_argument.arguments = vec!["cargo  test".to_owned()];
    let argument_error = compare(&changed_argument).expect_err("argv strings must remain exact");
    assert!(
        argument_error.starts_with("oracle argv mismatch at invocation 1"),
        "argv whitespace difference was not identified: {argument_error}"
    );
    let mut changed_presence = baseline.clone();
    changed_presence.rust_flags_present = true;
    let presence_error = compare(&changed_presence).expect_err("presence should differ");
    assert!(
        presence_error.starts_with("oracle RUST_FLAGS presence mismatch at invocation 1"),
        "RUST_FLAGS presence difference was not identified: {presence_error}"
    );
}

/// Checks default, environment, command-line and platform flag routing in live Make branches.
#[rstest]
#[case::make_test(RustFlagsInput::Default)]
#[case::make_test_environment(RustFlagsInput::Environment)]
#[case::make_test_command_line(RustFlagsInput::CommandLine)]
fn make_test_inputs_are_unexported_and_restate_recipe_flags(
    #[case] input: RustFlagsInput,
) -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        for host in [Host::Linux, Host::Darwin] {
            let route = |target, nextest_available, nextest_flags| RunRequest {
                target,
                host,
                rust_flags: input,
                nextest_available,
                nextest_flags,
                failure: "",
            };
            for target in ["test", "lint", "typecheck", "build", "release"] {
                make_input::check_target(scratch, &route(target, true, ""))?;
            }
            for target in ["test", "test-ci", "test-quick"] {
                make_input::check_target(scratch, &route(target, true, NEXTEST_OPTIONS))?;
            }
            make_input::check_target(scratch, &route("test", false, NEXTEST_OPTIONS))?;
        }
        Ok(())
    })
}

/// Requires controlled Nextest, fallback Cargo and doctest failures to fail Make.
#[rstest]
#[case::make_test_nextest_failure("test", true, "nextest-run")]
#[case::make_test_fallback_failure("test", false, "cargo-test")]
#[case::make_test_doctest_failure("test", true, "doctest")]
#[case::make_test_ci_failure("test-ci", true, "nextest-run")]
#[case::make_test_quick_failure("test-quick", true, "nextest-run")]
fn make_test_propagates_each_leaf_failure(
    #[case] target: &'static str,
    #[case] nextest_available: bool,
    #[case] failure: &'static str,
) -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        let observation = scratch.run(&RunRequest {
            target,
            host: Host::Linux,
            rust_flags: RustFlagsInput::CommandLine,
            nextest_available,
            nextest_flags: NEXTEST_OPTIONS,
            failure,
        })?;
        failure_contract(&observation, failure)
    })
}

/// Proves the source-copy contract rejects representative Makefile mutations.
#[rstest]
#[case::mutation_unexport("unexport")]
#[case::mutation_nextest_option("nextest-option")]
#[case::mutation_fallback_leak("fallback-leak")]
#[case::mutation_restatement("restatement")]
#[case::mutation_failure_mask("failure-mask")]
fn make_test_contract_detects_source_mutations(
    #[case] mutation: &'static str,
) -> Result<(), String> {
    make_input::with_scratch(|scratch| check_source_mutation(scratch, mutation))
}

/// Runs one source mutation only after its unmodified production route passes.
fn check_source_mutation(scratch: &make_input::Scratch, mutation: &str) -> Result<(), String> {
    let baseline = mutation_request(mutation, "")?;
    make_input::check_target(scratch, &baseline)?;
    let expected = expected_calls(&baseline)?;
    if mutation == "failure-mask" {
        check_failure_mask_baseline(scratch)?;
    }
    let changed = mutated_makefile(mutation, &scratch.read_makefile()?)?;
    scratch.write_makefile(&changed)?;
    let failure = if mutation == "failure-mask" {
        "nextest-run"
    } else {
        ""
    };
    let observation = scratch.run(&mutation_request(mutation, failure)?)?;
    check_mutated_route(mutation, &expected, &observation)
}

/// Selects the literal Make route and Nextest branch for one finite mutation name.
fn mutation_request<'a>(mutation: &str, failure: &'a str) -> Result<RunRequest<'a>, String> {
    let (target, nextest_available) = match mutation {
        "unexport" | "nextest-option" => ("test-ci", true),
        "fallback-leak" => ("test", false),
        "restatement" => ("typecheck", true),
        "failure-mask" => ("test", true),
        _ => return Err(format!("unknown Makefile mutation {mutation}")),
    };
    Ok(RunRequest {
        target,
        host: Host::Linux,
        rust_flags: RustFlagsInput::CommandLine,
        nextest_available,
        nextest_flags: NEXTEST_OPTIONS,
        failure,
    })
}

/// Applies exactly one uniquely located mutation to the copied candidate Makefile.
fn mutated_makefile(mutation: &str, original: &str) -> Result<String, String> {
    match mutation {
        "unexport" => replace_once(original, "unexport RUST_FLAGS\n", ""),
        "nextest-option" => replace_once(
            original,
            "nextest run --profile ci $(TEST_FLAGS) $(NEXTEST_FLAGS) $(BUILD_JOBS)",
            "nextest run --profile ci $(TEST_FLAGS) $(BUILD_JOBS)",
        ),
        "fallback-leak" => replace_once(
            original,
            "$(CARGO) test $(TEST_FLAGS) $(BUILD_JOBS); \\",
            "$(CARGO) test $(TEST_FLAGS) $(NEXTEST_FLAGS) $(BUILD_JOBS); \\",
        ),
        "restatement" => replace_once(
            original,
            "$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)\" $(CARGO) check",
            "$(STANDARD_RUSTFLAGS)\" $(CARGO) check",
        ),
        "failure-mask" => replace_once(
            original,
            "$(CARGO) nextest run --profile default $(TEST_FLAGS) $(NEXTEST_FLAGS) $(BUILD_JOBS); \\",
            "$(CARGO) nextest run --profile default $(TEST_FLAGS) $(NEXTEST_FLAGS) $(BUILD_JOBS) || true; \\",
        ),
        _ => Err(format!("unknown Makefile mutation {mutation}")),
    }
}

/// Confirms the original recipe propagates the named failing Nextest leaf.
fn check_failure_mask_baseline(scratch: &make_input::Scratch) -> Result<(), String> {
    let observation = scratch.run(&mutation_request("failure-mask", "nextest-run")?)?;
    failure_contract(&observation, "nextest-run")
}

/// Rejects an unsuccessful mutated route before checking its one intended contract difference.
fn check_mutated_route(
    mutation: &str,
    expected: &[Invocation],
    observation: &Observation,
) -> Result<(), String> {
    if !observation.status.success() {
        return Err(format!("{mutation} mutation made its route fail"));
    }
    if mutation == "failure-mask" {
        check_failure_mask_mutation(expected, observation)
    } else {
        check_ordinary_mutation(mutation, expected, observation)
    }
}

/// Requires the swallowed-error mutation to reach its failing leaf and report only lost propagation.
fn check_failure_mask_mutation(
    expected: &[Invocation],
    observation: &Observation,
) -> Result<(), String> {
    super::require_calls(expected, &observation.calls, "failure-mask route")?;
    if !reached_failure(observation, "nextest-run") {
        return Err("failure-mask mutation missed the failing Nextest leaf".to_owned());
    }
    match failure_contract(observation, "nextest-run") {
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
    mutation: &str,
    expected: &[Invocation],
    observation: &Observation,
) -> Result<(), String> {
    let diagnostic = super::mutation_diagnostic(mutation, expected, &observation.calls)?;
    match super::require_calls(expected, &observation.calls, mutation) {
        Err(error) if error == diagnostic => {
            super::exact_mutation_delta(mutation, expected, &observation.calls)
        }
        Err(error) => Err(format!("{mutation} caused an unrelated rejection: {error}")),
        Ok(()) => Err(format!(
            "{mutation} was not detected by the baseline contract"
        )),
    }
}

/// Uses recursive Make to prove command-line values reach the real `test-ci` recipe.
#[rstest]
fn make_test_cli_flags_reach_recursive_production_recipe() -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        scratch.add_recursive_fixture()?;
        make_input::check_target(
            scratch,
            &RunRequest {
                target: "gauss-input-recursive",
                host: Host::Linux,
                rust_flags: RustFlagsInput::CommandLine,
                nextest_available: true,
                nextest_flags: NEXTEST_OPTIONS,
                failure: "",
            },
        )
    })
}
