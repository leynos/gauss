//! Pure expected-row queries for controlled Make invocation contracts.

use super::super::super::make_input_env::{
    self as make_input, INHERITED_RUSTDOCFLAGS, INHERITED_RUSTFLAGS, Invocation,
    RESTATED_RUSTDOCFLAGS, RunRequest, RustFlagsInput,
};
use super::super::RUST_FLAGS_INPUT;

/// Returns expected recorder rows for every invocation in one controlled Make route.
pub(crate) fn expected_calls(request: &RunRequest<'_>) -> Result<Vec<Invocation>, String> {
    let argument_rows = expected_arguments(request)?;
    Ok(argument_rows
        .into_iter()
        .enumerate()
        .map(|(index, row_arguments)| Invocation {
            arguments: row_arguments,
            rust_flags_present: false,
            rustflags: expected_rustflags(request, index),
            rustdocflags: expected_rustdocflags(request.target, index).to_owned(),
        })
        .collect())
}

/// Selects the inherited or recipe-restated rustdoc flags for one recorded call.
fn expected_rustdocflags(target: &str, index: usize) -> &'static str {
    match (target, index) {
        ("test", 2) | ("lint", 0) => RESTATED_RUSTDOCFLAGS,
        _ => INHERITED_RUSTDOCFLAGS,
    }
}

/// Returns expected argv groups for every command in one supported Make route.
fn expected_arguments(request: &RunRequest<'_>) -> Result<Vec<Vec<String>>, String> {
    let test_flags = "--workspace --all-targets --all-features";
    let jobs = "-j1";
    let make_flags = match request.rust_flags {
        RustFlagsInput::Default => "-D warnings",
        RustFlagsInput::Environment => "-D warnings --cfg=gauss_environment_value_loses_to_cli",
        RustFlagsInput::CommandLine => RUST_FLAGS_INPUT,
    };
    let groups: Vec<Vec<&str>> = match request.target {
        "test" => test_argument_groups(request, test_flags, jobs),
        "test-ci" | "gauss-input-recursive" | "test-quick" => {
            nextest_argument_groups(request, test_flags, jobs)
        }
        "lint" => vec![
            vec!["doc --workspace --no-deps"],
            vec!["clippy", test_flags, "--", make_flags],
            vec!["--all --", test_flags],
        ],
        "typecheck" => vec![vec!["check", test_flags]],
        "build" => vec![vec!["build", jobs]],
        "release" => vec![vec!["build", jobs, "--release"]],
        _ => {
            return Err(format!(
                "unsupported controlled Make target {}",
                request.target
            ));
        }
    };
    Ok(groups
        .iter()
        .map(|group| make_input::arguments(&group.join(" ")))
        .collect())
}

/// Groups the probe, selected test-suite command and doctest command in Make order.
fn test_argument_groups<'a>(
    request: &RunRequest<'a>,
    test_flags: &'a str,
    jobs: &'a str,
) -> Vec<Vec<&'a str>> {
    let suite = if request.nextest_available {
        vec![
            "nextest run --profile default",
            test_flags,
            request.nextest_flags,
            jobs,
        ]
    } else {
        vec!["test", test_flags, jobs]
    };
    vec![
        vec!["nextest --version"],
        suite,
        vec!["test --workspace --doc --all-features", jobs],
    ]
}

/// Selects the CI or quick Nextest invocation group without changing its arguments.
fn nextest_argument_groups<'a>(
    request: &RunRequest<'a>,
    test_flags: &'a str,
    jobs: &'a str,
) -> Vec<Vec<&'a str>> {
    let command = if request.target == "test-quick" {
        "nextest run --profile default --lib"
    } else {
        "nextest run --profile ci"
    };
    vec![vec![command, test_flags, request.nextest_flags, jobs]]
}

/// Returns effective compiler flags in recipe order for one recorded call.
fn expected_rustflags(request: &RunRequest<'_>, index: usize) -> String {
    let make_flags = match request.rust_flags {
        RustFlagsInput::Default => "-D warnings",
        RustFlagsInput::Environment => "-D warnings --cfg=gauss_environment_value_loses_to_cli",
        RustFlagsInput::CommandLine => "-D warnings --cfg=gauss_rust_flags_marker",
    };
    let takes_make_flags = match request.target {
        "test" => index != 0,
        "test-ci" | "test-quick" | "gauss-input-recursive" | "typecheck" => true,
        "lint" => index == 2,
        _ => false,
    };
    let mut flags = INHERITED_RUSTFLAGS.to_owned();
    if takes_make_flags {
        flags.push(' ');
        flags.push_str(make_flags);
    }
    if request.target != "release" && request.host.takes_linker_flag() {
        flags.push_str(" -Clink-arg=-fuse-ld=mold");
    }
    flags
}
