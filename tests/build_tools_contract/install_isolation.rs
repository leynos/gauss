//! Independent Bash isolation procedures for pinned build-tool entrypoints.
//!
//! The named rstest cases stay in `install.rs`; these helpers keep their full
//! process observations beside the scenario setup.

use super::{
    Entry, InjectedFunction, InjectionMode, Probe, ProbeRequest, Request,
    install_oracles as oracle, support::with_scratch,
};

/// Verifies unexpected arguments stop before any external tool call.
pub(super) fn entrypoint_rejects_unexpected_arguments_before_tool_calls(
    entry: Entry,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(entry);
        request.controls.unexpected_argument = true;
        let expected = oracle::expected(
            1,
            Vec::new(),
            Some("build-tools: unexpected arguments".to_owned()),
            oracle::prefix_state(false, false),
        );
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

/// Compares clean and injected entrypoint traces and raw observations.
pub(super) fn exported_functions_do_not_shadow_entrypoint_tools_or_builtins(
    entry: Entry,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let clean = oracle::successful_entry(scratch, &Request::new(entry))?;
        let mut request = Request::new(entry);
        request.controls.injection = InjectionMode::EntryFunctions;
        let injected = oracle::successful_entry(scratch, &request)?;
        oracle::compare_clean_injected(scratch, &clean, &injected)
    })
}

/// Checks direct privileged fake execution ignores imported functions.
pub(super) fn privileged_fake_shebang_disables_function_import(probe: Probe) -> Result<(), String> {
    with_scratch(|scratch| {
        let (functions, calls, stdout, prefix) = match probe {
            Probe::FakeCurl => (
                vec![InjectedFunction::Printf],
                vec![oracle::call(
                    "curl",
                    &[
                        "--fail",
                        "--silent",
                        "--show-error",
                        "--location",
                        "--connect-timeout",
                        "15",
                        "--speed-limit",
                        "1024",
                        "--speed-time",
                        "60",
                        "--output",
                        scratch.path.join("probe-archive").as_str(),
                        "https://example.invalid/fixture.tar.gz",
                    ],
                    0,
                )],
                Vec::new(),
                oracle::prefix_state(false, false),
            ),
            Probe::InstalledMold => {
                oracle::successful_entry(scratch, &Request::new(Entry::Install))?;
                (
                    vec![InjectedFunction::Printf, InjectedFunction::Mold],
                    vec![oracle::call("prefix-mold", &["--version"], 0)],
                    b"mold 2.41.0\n".to_vec(),
                    oracle::prefix_state(true, true),
                )
            }
            _ => return Err("unexpected privileged probe identifier".to_owned()),
        };
        let request = ProbeRequest { probe, functions };
        let mut expected = oracle::expected(0, calls, None, prefix);
        expected.stdout = stdout;
        let observation = scratch.probe(&request)?;
        oracle::verify_observation(scratch, &observation, expected)?;
        if probe == Probe::FakeCurl {
            let archive = scratch.read_relative("probe-archive")?;
            oracle::require(
                archive == oracle::INERT_ARCHIVE,
                "inert archive bytes",
                &format!("{:?}", oracle::INERT_ARCHIVE),
                &format!("{archive:?}"),
            )?;
        }
        Ok(())
    })
}

/// Checks privileged entrypoints ignore startup-file command injection.
pub(super) fn privileged_entrypoints_ignore_startup_file_injection(
    entry: Entry,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let clean = oracle::successful_entry(scratch, &Request::new(entry))?;
        let mut request = Request::new(entry);
        request.controls.injection = InjectionMode::StartupFiles;
        let injected = oracle::successful_entry(scratch, &request)?;
        oracle::compare_clean_injected(scratch, &clean, &injected)
    })
}

/// Confirms unprivileged controls import the selected function fixture.
pub(super) fn unprivileged_controls_prove_function_fixture_import(
    probe: Probe,
    function: InjectedFunction,
    marker: &[u8],
) -> Result<(), String> {
    with_scratch(|scratch| {
        let request = ProbeRequest {
            probe,
            functions: vec![function],
        };
        let observation = scratch.probe(&request)?;
        let mut expected =
            oracle::expected(97, Vec::new(), None, oracle::prefix_state(false, false));
        expected.shadow_marker = marker.to_vec();
        oracle::verify_observation(scratch, &observation, expected)
    })
}
