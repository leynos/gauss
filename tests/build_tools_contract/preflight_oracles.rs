//! Independent preflight trace builders and availability scenarios.
//!
//! These helpers retain literal expectations for the named cases in `preflight.rs`.

use super::{
    Entry, Failure, Invocation, Program, Request, install_oracles as oracle,
    support::{Scratch, with_scratch},
};

/// Selects the first or second exact invocation matching a tool and argv.
#[derive(Clone, Copy)]
pub(super) enum CallOccurrence {
    /// Selects the first matching invocation.
    First,
    /// Selects the second matching invocation.
    Second,
}

/// Describes an invocation by its complete tool, argument vector, and ordinal.
pub(super) struct InvocationSelection<'a> {
    /// The executable recorded by the fake process trace.
    pub(super) tool: &'a str,
    /// The complete argv recorded by the fake process trace.
    pub(super) arguments: &'a [&'a str],
    /// Which exact matching invocation to select.
    pub(super) occurrence: CallOccurrence,
}

/// Describes which independently expected later preflight stages are reached.
pub(super) enum FailureTrace {
    /// Clang's version query fails before its syntax probe.
    ClangVersionOnly,
    /// The complete expected preflight trace is reached.
    Complete,
    /// The declared toolchain-list query fails before its component query.
    MissingDeclaredComponentQuery,
    /// The stable toolchain-list query fails before its component query.
    MissingStableComponentQuery,
}

/// Binds an independent failure, exact process selection, trace, and diagnostic.
pub(super) struct CheckFailureCase {
    /// The controlled production stage that must fail.
    pub(super) failure: Failure,
    /// The exact process invocation whose fake status is changed.
    pub(super) selection: InvocationSelection<'static>,
    /// Which later stages are omitted from the independently built trace.
    pub(super) trace: FailureTrace,
    /// The exact diagnostic expected from the failed stage.
    pub(super) diagnostic: &'static str,
}

/// Describes one named missing-requirement scenario without string dispatch.
#[derive(Clone, Copy)]
pub(super) enum MissingRequirement {
    /// The pinned mold executable is absent.
    MoldAbsent,
    /// The pinned mold executable cannot report its version.
    MoldUnusable,
    /// The installed mold version does not match its pin.
    MoldWrongPin,
    /// The configured clang executable is absent.
    ClangAbsent,
    /// The declared Rust toolchain is absent.
    DeclaredToolchainAbsent,
    /// A required component is absent from the declared toolchain.
    DeclaredComponentAbsent,
    /// The stable toolchain is absent.
    StableToolchainAbsent,
    /// Rustfmt is absent from the stable toolchain.
    StableRustfmtAbsent,
}

/// Replaces one independently selected invocation status without changing its argv.
pub(super) fn set_call_status(
    calls: &mut [Invocation],
    selection: &InvocationSelection<'_>,
    status: i32,
) -> Result<(), String> {
    let ordinal = match selection.occurrence {
        CallOccurrence::First => 0,
        CallOccurrence::Second => 1,
    };
    let call = calls
        .iter_mut()
        .filter(|call| {
            call.tool == selection.tool
                && call
                    .argv
                    .iter()
                    .map(String::as_str)
                    .eq(selection.arguments.iter().copied())
        })
        .nth(ordinal)
        .ok_or_else(|| match selection.occurrence {
            CallOccurrence::First => format!(
                "expected {} call with argv {:?}",
                selection.tool, selection.arguments
            ),
            CallOccurrence::Second => format!(
                "expected second {} call with argv {:?}",
                selection.tool, selection.arguments
            ),
        })?;
    call.status = status;
    Ok(())
}

/// Identifies the exact Rustup component inventory query in an expected trace.
fn is_component_query(call: &Invocation) -> bool {
    matches!(
        call.argv.as_slice(),
        [command, action, ..] if command == "component" && action == "list"
    )
}

/// Removes one ordinal component query from an independently built trace.
pub(super) fn skip_component_query(
    calls: &mut Vec<Invocation>,
    ordinal: usize,
) -> Result<(), String> {
    let mut query_count = 0;
    calls.retain(|call| {
        if is_component_query(call) {
            query_count += 1;
            query_count != ordinal
        } else {
            true
        }
    });
    if ordinal == 0 || query_count < ordinal {
        return Err(format!("expected component query ordinal {ordinal}"));
    }
    Ok(())
}

/// Applies the explicitly named stage omission to an independent expected trace.
fn prepare_failure_trace(
    calls: &mut Vec<Invocation>,
    case: &CheckFailureCase,
) -> Result<(), String> {
    match &case.trace {
        FailureTrace::ClangVersionOnly => calls.retain(|call| {
            call.tool != "clang"
                || call
                    .argv
                    .iter()
                    .map(String::as_str)
                    .eq(case.selection.arguments.iter().copied())
        }),
        FailureTrace::Complete => {}
        FailureTrace::MissingDeclaredComponentQuery => skip_component_query(calls, 1)?,
        FailureTrace::MissingStableComponentQuery => skip_component_query(calls, 2)?,
    }
    Ok(())
}

/// Verifies a named controlled preflight failure against its complete trace.
pub(super) fn verify_check_failure(case: &CheckFailureCase) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.controls.failure = Some(case.failure);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        prepare_failure_trace(&mut calls, case)?;
        set_call_status(&mut calls, &case.selection, 97)?;

        let mut expected = oracle::expected(
            1,
            calls,
            Some(case.diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        if !matches!(&case.trace, FailureTrace::ClangVersionOnly) {
            expected.clang_stdin = oracle::CLANG_SOURCE.to_vec();
        }
        let observation = scratch.run(&request)?;
        oracle::verify_observation(scratch, &observation, expected)
    })
}

/// Verifies a pin or toolchain parse failure before host probes.
pub(super) fn verify_boot_failure(
    scratch: &Scratch,
    entry: Entry,
    diagnostic: &str,
) -> Result<(), String> {
    let request = Request::new(entry);
    let calls = oracle::boot(scratch, entry);
    verify_input_failure(scratch, &request, calls, diagnostic)
}

/// Verifies a host-input failure before any compiler or Rustup process.
pub(super) fn verify_host_failure(
    scratch: &Scratch,
    request: &Request,
    diagnostic: &str,
) -> Result<(), String> {
    let calls = oracle::host_calls(scratch, request);
    verify_input_failure(scratch, request, calls, diagnostic)
}

/// Verifies an input failure before compiler or Rustup work and its full trace.
fn verify_input_failure(
    scratch: &Scratch,
    request: &Request,
    calls: Vec<Invocation>,
    diagnostic: &str,
) -> Result<(), String> {
    let expected = oracle::expected(
        1,
        calls,
        Some(diagnostic.to_owned()),
        oracle::prefix_state(false, false),
    );
    oracle::verify_observation(scratch, &scratch.run(request)?, expected)
}

/// Independently checks one install followed by its channel-specific preflight.
pub(super) fn verify_install_then_check(
    scratch: &Scratch,
    request: &mut Request,
    channel: &str,
    extras: &[&str],
) -> Result<(), String> {
    let mut install = oracle::install_expectation("x86_64", channel)?;
    install.extras = extras;
    let mut expected_install = oracle::expected(
        0,
        oracle::install_calls(scratch, request, &install),
        Some("build-tools: ready; verify with:".to_owned()),
        oracle::prefix_state(true, true),
    );
    expected_install.sha_stdin = oracle::checksum_stdin(&install.digest, &install.archive);
    oracle::verify_observation(scratch, &scratch.run(request)?, expected_install)?;

    request.entry = Entry::Check;
    let mut expected_check = oracle::expected(
        0,
        oracle::check_calls(scratch, request, channel),
        Some("build-tools: clang is usable at".to_owned()),
        oracle::prefix_state(true, true),
    );
    expected_check.clang_stdin = oracle::CLANG_SOURCE.to_vec();
    oracle::verify_observation(scratch, &scratch.run(request)?, expected_check)
}

/// Runs one named missing-tool scenario and checks its complete trace.
pub(super) fn verify_missing_requirements(
    scenario: MissingRequirement,
    diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        match scenario {
            MissingRequirement::MoldAbsent => {
                request.controls.unavailable.push(Program::Mold);
                calls.retain(|call| call.tool != "mold");
            }
            MissingRequirement::MoldUnusable => {
                request.controls.failure = Some(Failure::MoldVersion);
                set_call_status(
                    &mut calls,
                    &InvocationSelection {
                        tool: "mold",
                        arguments: &["--version"],
                        occurrence: CallOccurrence::First,
                    },
                    97,
                )?;
            }
            MissingRequirement::MoldWrongPin => {
                request.controls.mold_version_output = Some("mold 0.0.0".to_owned());
            }
            MissingRequirement::ClangAbsent => {
                request.controls.unavailable.push(Program::Clang);
                calls.retain(|call| call.tool != "clang");
            }
            MissingRequirement::DeclaredToolchainAbsent => {
                let [declared, _stable] = &mut request.controls.rustup.list_output;
                declared.clear();
                skip_component_query(&mut calls, 1)?;
            }
            MissingRequirement::DeclaredComponentAbsent => {
                let [declared, _stable] = &mut request.controls.rustup.component_output;
                "rustfmt (installed)\nclippy (installed)\n".clone_into(declared);
            }
            MissingRequirement::StableToolchainAbsent => {
                let [_declared, stable] = &mut request.controls.rustup.list_output;
                stable.clear();
                skip_component_query(&mut calls, 2)?;
            }
            MissingRequirement::StableRustfmtAbsent => {
                let [_declared, stable] = &mut request.controls.rustup.component_output;
                stable.clear();
            }
        }
        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        if !matches!(scenario, MissingRequirement::ClangAbsent) {
            expected.clang_stdin = oracle::CLANG_SOURCE.to_vec();
        }
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

/// Verifies both independent Rustup availability checks when the fake is absent.
pub(super) fn verify_missing_rustup() -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.controls.unavailable.push(Program::Rustup);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        calls.retain(|call| call.tool != "rustup");
        let diagnostic = "rustup not found on PATH; install it from https://rustup.rs";
        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        expected.clang_stdin = oracle::CLANG_SOURCE.to_vec();

        let observation = scratch.run(&request)?;
        let diagnostic_count = String::from_utf8_lossy(&observation.stderr)
            .matches(diagnostic)
            .count();
        oracle::require(
            diagnostic_count == 2,
            "missing Rustup diagnostic count",
            "2",
            &diagnostic_count.to_string(),
        )?;
        oracle::verify_observation(scratch, &observation, expected)
    })
}
