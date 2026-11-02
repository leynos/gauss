//! Thin isolation and missing-tool cases that delegate to the detailed procedures.
//!
//! Keeping these delegating cases out of `install` leaves that module inside the
//! 400-line budget after the pinned formatter expanded its call sites.

use rstest::rstest;

use super::{
    Entry,
    InjectedFunction,
    Probe,
    Program,
    install_isolation,
    install_oracles as oracle,
};

#[rstest]
#[case::install(Entry::Install)]
#[case::check(Entry::Check)]
fn entrypoint_rejects_unexpected_arguments_before_tool_calls(
    #[case] entry: Entry,
) -> Result<(), String> {
    install_isolation::entrypoint_rejects_unexpected_arguments_before_tool_calls(entry)
}

#[rstest]
#[case::install(Entry::Install)]
#[case::check(Entry::Check)]
fn exported_functions_do_not_shadow_entrypoint_tools_or_builtins(
    #[case] entry: Entry,
) -> Result<(), String> {
    install_isolation::exported_functions_do_not_shadow_entrypoint_tools_or_builtins(entry)
}

#[rstest]
#[case::curl_executable(Probe::FakeCurl)]
#[case::installed_prefix_mold(Probe::InstalledMold)]
fn privileged_fake_shebang_disables_function_import(#[case] probe: Probe) -> Result<(), String> {
    install_isolation::privileged_fake_shebang_disables_function_import(probe)
}

#[rstest]
#[case::install(Entry::Install)]
#[case::check(Entry::Check)]
fn privileged_entrypoints_ignore_startup_file_injection(
    #[case] entry: Entry,
) -> Result<(), String> {
    install_isolation::privileged_entrypoints_ignore_startup_file_injection(entry)
}

#[rstest]
#[case::curl_import(Probe::CurlImportControl, InjectedFunction::Curl, b"curl\n".as_slice())]
#[case::printf_import(Probe::PrintfImportControl, InjectedFunction::Printf, b"printf\n".as_slice())]
fn unprivileged_controls_prove_function_fixture_import(
    #[case] probe: Probe,
    #[case] function: InjectedFunction,
    #[case] marker: &[u8],
) -> Result<(), String> {
    install_isolation::unprivileged_controls_prove_function_fixture_import(probe, function, marker)
}

#[rstest]
#[case::missing_curl(Program::Curl, false)]
#[case::missing_sha256sum(Program::Sha256sum, false)]
#[case::missing_tar(Program::Tar, false)]
#[case::missing_rustup(Program::Rustup, true)]
fn installer_requires_each_external_tool(
    #[case] unavailable: Program,
    #[case] after_mold: bool,
) -> Result<(), String> {
    oracle::verify_missing_installer_tool(unavailable, after_mold)
}
