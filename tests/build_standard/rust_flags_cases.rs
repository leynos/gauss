//! Exercises Rust flag and Nextest routing through the copied production Makefile.

use rstest::rstest;

use super::super::{
    make::Host,
    make_input_env::{
        self as make_input,
        Invocation,
        NEXTEST_OPTIONS,
        Observation,
        RunRequest,
        RustFlagsInput,
    },
};

/// Pure expected-row queries for supported Make routes.
#[path = "rust_flags_expected.rs"]
mod expected;

/// Re-exports the expected-row query through the established cases module.
pub(crate) use expected::expected_calls;

/// The finite source mutations supported by the copied-Makefile contracts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mutation {
    Unexport,
    NextestOption,
    FallbackLeak,
    Restatement,
    FailureMask,
}

impl std::fmt::Display for Mutation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Unexport => "unexport",
            Self::NextestOption => "nextest-option",
            Self::FallbackLeak => "fallback-leak",
            Self::Restatement => "restatement",
            Self::FailureMask => "failure-mask",
        })
    }
}

/// The fake command failures understood by the recorder protocol.
#[derive(Clone, Copy)]
pub(super) enum FailureCase {
    NextestRun,
    CargoTest,
    Doctest,
    Clippy,
    Whitaker,
}

impl FailureCase {
    /// Returns the recorder protocol token; for example, `Self::Doctest` yields `"doctest"`.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NextestRun => "nextest-run",
            Self::CargoTest => "cargo-test",
            Self::Doctest => "doctest",
            Self::Clippy => "clippy",
            Self::Whitaker => "whitaker",
        }
    }
}

/// Requires Make to fail after the named fake leaf command returns non-zero.
pub(super) fn failure_contract(
    observation: &Observation,
    failure: FailureCase,
) -> Result<(), String> {
    if !observation.status.success() && reached_failure(observation, failure) {
        Ok(())
    } else {
        Err(format!(
            "make did not propagate the controlled {} failure",
            failure.as_str()
        ))
    }
}

/// Reports whether the named fake failing leaf was recorded.
pub(super) fn reached_failure(observation: &Observation, failure: FailureCase) -> bool {
    observation.calls.iter().any(|call| match failure {
        FailureCase::NextestRun => {
            call.arguments.first().map(String::as_str) == Some("nextest")
                && call.arguments.get(1).map(String::as_str) == Some("run")
        }
        FailureCase::CargoTest => {
            call.arguments.first().map(String::as_str) == Some("test")
                && !call.arguments.iter().any(|argument| argument == "--doc")
        }
        FailureCase::Doctest => {
            call.arguments.first().map(String::as_str) == Some("test")
                && call.arguments.iter().any(|argument| argument == "--doc")
        }
        FailureCase::Clippy => call.arguments.first().map(String::as_str) == Some("clippy"),
        FailureCase::Whitaker => call.arguments.first().map(String::as_str) == Some("--all"),
    })
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
            for target in [
                "test",
                "lint",
                "lint-clippy",
                "lint-whitaker",
                "typecheck",
                "build",
                "release",
            ] {
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
#[case::make_test_nextest_failure("test", true, FailureCase::NextestRun)]
#[case::make_test_fallback_failure("test", false, FailureCase::CargoTest)]
#[case::make_test_doctest_failure("test", true, FailureCase::Doctest)]
#[case::make_test_ci_failure("test-ci", true, FailureCase::NextestRun)]
#[case::make_test_quick_failure("test-quick", true, FailureCase::NextestRun)]
fn make_test_propagates_each_leaf_failure(
    #[case] target: &'static str,
    #[case] nextest_available: bool,
    #[case] failure: FailureCase,
) -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        let observation = scratch.run(&RunRequest {
            target,
            host: Host::Linux,
            rust_flags: RustFlagsInput::CommandLine,
            nextest_available,
            nextest_flags: NEXTEST_OPTIONS,
            failure: failure.as_str(),
        })?;
        failure_contract(&observation, failure)
    })
}

/// Requires each lint leaf, and the composite gate, to fail Make when the leaf fails.
#[rstest]
#[case::make_lint_clippy_failure("lint", FailureCase::Clippy)]
#[case::make_lint_whitaker_failure("lint", FailureCase::Whitaker)]
#[case::make_lint_clippy_leaf_failure("lint-clippy", FailureCase::Clippy)]
#[case::make_lint_whitaker_leaf_failure("lint-whitaker", FailureCase::Whitaker)]
fn make_lint_propagates_each_leaf_failure(
    #[case] target: &'static str,
    #[case] failure: FailureCase,
) -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        let observation = scratch.run(&lint_request(target, failure.as_str()))?;
        failure_contract(&observation, failure)
    })
}

/// A failing Clippy leaf must stop the composite gate before Whitaker runs.
#[test]
fn make_lint_skips_whitaker_after_clippy_failure() -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        let observation = scratch.run(&lint_request("lint", FailureCase::Clippy.as_str()))?;
        if observation.status.success() {
            return Err("make lint succeeded after a controlled Clippy failure".to_owned());
        }
        if observation.calls.iter().any(is_whitaker_call) {
            return Err("make lint reached Whitaker after Clippy failed".to_owned());
        }
        Ok(())
    })
}

/// The composite gate keeps Clippy ahead of Whitaker when Make runs in parallel.
#[test]
fn make_lint_orders_clippy_before_whitaker_under_parallel_make() -> Result<(), String> {
    make_input::with_scratch(|scratch| {
        let observation = scratch.run_with_jobs(&lint_request("lint", ""), 4)?;
        let clippy_at = observation.calls.iter().rposition(is_clippy_call);
        let whitaker_at = observation.calls.iter().position(is_whitaker_call);
        match (clippy_at, whitaker_at) {
            (Some(clippy_index), Some(whitaker_index)) if clippy_index < whitaker_index => Ok(()),
            _ => Err(format!(
                "make -j4 lint did not order Clippy before Whitaker: {:?}",
                observation
                    .calls
                    .iter()
                    .map(|call| call.arguments.first().cloned().unwrap_or_default())
                    .collect::<Vec<_>>()
            )),
        }
    })
}

/// Builds the controlled request for one lint goal and its injected failure.
fn lint_request(target: &'static str, failure: &'static str) -> RunRequest<'static> {
    RunRequest {
        target,
        host: Host::Linux,
        rust_flags: RustFlagsInput::CommandLine,
        nextest_available: true,
        nextest_flags: "",
        failure,
    }
}

/// Reports whether a recorded call is the Clippy leaf.
fn is_clippy_call(call: &Invocation) -> bool {
    call.arguments.first().map(String::as_str) == Some("clippy")
}

/// Reports whether a recorded call is the Whitaker suite.
fn is_whitaker_call(call: &Invocation) -> bool {
    call.arguments.first().map(String::as_str) == Some("--all")
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
