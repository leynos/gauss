//! Independent preflight trace builders and availability scenarios.
//!
//! These helpers retain literal expectations for the named cases in `preflight.rs`.

use super::{
    Entry, Failure, Invocation, Program, Request, install_oracles as oracle,
    support::{Scratch, with_scratch},
};

/// Replaces one independently selected invocation status without changing its argv.
pub(super) fn set_call_status(
    calls: &mut [Invocation],
    tool: &str,
    arguments: &[&str],
    status: i32,
) -> Result<(), String> {
    let call = calls
        .iter_mut()
        .find(|call| {
            call.tool == tool
                && call
                    .argv
                    .iter()
                    .map(String::as_str)
                    .eq(arguments.iter().copied())
        })
        .ok_or_else(|| format!("expected {tool} call with argv {arguments:?}"))?;
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

/// Verifies a pin or toolchain parse failure before host probes.
pub(super) fn verify_boot_failure(
    scratch: &Scratch,
    entry: Entry,
    diagnostic: &str,
) -> Result<(), String> {
    let expected = oracle::expected(
        1,
        oracle::boot(scratch, entry),
        Some(diagnostic.to_owned()),
        oracle::prefix_state(false, false),
    );
    oracle::verify_observation(scratch, &scratch.run(&Request::new(entry))?, expected)
}

/// Verifies a host-input failure before any compiler or Rustup process.
pub(super) fn verify_host_failure(
    scratch: &Scratch,
    request: &Request,
    diagnostic: &str,
) -> Result<(), String> {
    let expected = oracle::expected(
        1,
        oracle::host_calls(scratch, request),
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

/// Runs independently specified missing-tool scenarios and checks their complete traces.
pub(super) fn verify_missing_requirements(scenario: &str, diagnostic: &str) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        match scenario {
            "mold_absent" => {
                request.controls.unavailable.push(Program::Mold);
                calls.retain(|call| call.tool != "mold");
            }
            "mold_unusable" => {
                request.controls.failure = Some(Failure::MoldVersion);
                set_call_status(&mut calls, "mold", &["--version"], 97)?;
            }
            "mold_wrong_pin" => {
                request.controls.mold_version_output = Some("mold 0.0.0".to_owned());
            }
            "clang_absent" => {
                request.controls.unavailable.push(Program::Clang);
                calls.retain(|call| call.tool != "clang");
            }
            "declared_toolchain_absent" => {
                let [declared, _stable] = &mut request.controls.rustup.list_output;
                declared.clear();
                skip_component_query(&mut calls, 1)?;
            }
            "declared_component_absent" => {
                let [declared, _stable] = &mut request.controls.rustup.component_output;
                "rustfmt (installed)\nclippy (installed)\n".clone_into(declared);
            }
            "stable_toolchain_absent" => {
                let [_declared, stable] = &mut request.controls.rustup.list_output;
                stable.clear();
                skip_component_query(&mut calls, 2)?;
            }
            "stable_rustfmt_absent" => {
                let [_declared, stable] = &mut request.controls.rustup.component_output;
                stable.clear();
            }
            _ => return Err(format!("unknown preflight scenario {scenario}")),
        }
        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        if scenario != "clang_absent" {
            expected.clang_stdin = oracle::CLANG_SOURCE.to_vec();
        }
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

/// Marks the second exact matching call without changing its argument vector.
pub(super) fn set_second_matching_call_status(
    calls: &mut [Invocation],
    tool: &str,
    arguments: &[&str],
    status: i32,
) -> Result<(), String> {
    let call = calls
        .iter_mut()
        .filter(|call| {
            call.tool == tool
                && call
                    .argv
                    .iter()
                    .map(String::as_str)
                    .eq(arguments.iter().copied())
        })
        .nth(1)
        .ok_or_else(|| format!("expected second {tool} call with argv {arguments:?}"))?;
    call.status = status;
    Ok(())
}

/// Verifies the second stable Rustup query failure against its full trace.
pub(super) fn verify_stable_discovery_failure(
    failure: Failure,
    arguments: &[&str],
    skip_stable_component: bool,
    diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.controls.failure = Some(failure);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        if skip_stable_component {
            skip_component_query(&mut calls, 2)?;
        }
        set_second_matching_call_status(&mut calls, "rustup", arguments, 97)?;

        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        expected.clang_stdin = oracle::CLANG_SOURCE.to_vec();
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
