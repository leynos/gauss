//! Independent install and isolation contracts for pinned build tools.
//!
//! Each named story binds a planned C1 semantic ID to complete process evidence.

use rstest::rstest;

use super::{
    Entry, EnvironmentChange, Failure, Host, InjectedFunction, PayloadMode, Probe, Program,
    Request, install_isolation, install_oracles as oracle, support::with_scratch,
};

#[rstest]
#[case::x86_64("x86_64")]
#[case::amd64("amd64")]
#[case::aarch64("aarch64")]
#[case::arm64("arm64")]
#[case::armv7l("armv7l")]
#[case::armv8l("armv8l")]
#[case::riscv64("riscv64")]
#[case::ppc64le("ppc64le")]
#[case::s390x("s390x")]
#[case::loongarch64("loongarch64")]
fn linux_uname_selects_one_published_archive(#[case] machine: &str) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.machine = machine.to_owned();
        oracle::successful_entry(scratch, &request)?;
        Ok(())
    })
}

#[rstest]
#[case::bare_arm("arm")]
#[case::armv6l("armv6l")]
#[case::unpublished("mips64")]
fn unsupported_linux_machine_stops_before_side_effects(
    #[case] machine: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.machine = machine.to_owned();
        let calls = oracle::host_calls(scratch, &request);
        let diagnostic = format!("unsupported architecture for the pinned mold release: {machine}");
        let expected = oracle::expected(
            1,
            calls,
            Some(diagnostic),
            oracle::prefix_state(false, false),
        );
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::darwin(Host::Darwin)]
#[case::freebsd(Host::FreeBsd)]
fn non_linux_provisions_and_checks_rust_without_mold(#[case] host: Host) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut install = Request::new(Entry::Install);
        install.host = host;
        oracle::successful_entry(scratch, &install)?;
        let mut check = Request::new(Entry::Check);
        check.host = host;
        oracle::successful_entry(scratch, &check)?;
        Ok(())
    })
}

#[rstest]
#[case::system_failure(Failure::UnameSystem, "could not read host operating system", 3)]
#[case::linux_machine_failure(Failure::UnameMachine, "could not read host architecture", 4)]
fn uname_failure_fails_closed(
    #[case] failure: Failure,
    #[case] diagnostic: &str,
    #[case] call_count: usize,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.controls.failure = Some(failure);
        let mut calls = oracle::host_calls(scratch, &request);
        calls.truncate(call_count);
        calls
            .last_mut()
            .ok_or_else(|| "expected uname call is absent".to_owned())?
            .status = 97;
        let expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::linux_zero(
    Host::Linux,
    "0",
    "BUILD_TOOLS_MOLD_REQUIRED=0 cannot disable mold on Linux",
    4
)]
#[case::linux_invalid(Host::Linux, "invalid", "BUILD_TOOLS_MOLD_REQUIRED must be 0 or 1", 3)]
#[case::non_linux_invalid(
    Host::FreeBsd,
    "invalid",
    "BUILD_TOOLS_MOLD_REQUIRED must be 0 or 1",
    3
)]
fn required_override_cannot_skip_linux(
    #[case] host: Host,
    #[case] value: &str,
    #[case] diagnostic: &str,
    #[case] call_count: usize,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.host = host;
        request.controls.environment.push(EnvironmentChange {
            name: "BUILD_TOOLS_MOLD_REQUIRED".to_owned(),
            value: Some(value.to_owned()),
        });
        let mut calls = oracle::host_calls(scratch, &request);
        calls.truncate(call_count);
        let expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::missing_file(
    None,
    "expected exactly one checksum for mold-2.41.0-x86_64-linux.tar.gz; checksum file is missing"
)]
#[case::missing_selected(
    Some(""),
    "expected exactly one checksum for mold-2.41.0-x86_64-linux.tar.gz in"
)]
#[case::duplicate_selected(Some(concat!("a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d  mold-2.41.0-x86_64-linux.tar.gz\n", "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d  mold-2.41.0-x86_64-linux.tar.gz\n")), "expected exactly one checksum for mold-2.41.0-x86_64-linux.tar.gz in")]
#[case::malformed_digest(
    Some("not-a-digest  mold-2.41.0-x86_64-linux.tar.gz\n"),
    "invalid SHA-256 checksum recorded for mold-2.41.0-x86_64-linux.tar.gz in"
)]
#[case::extra_field(
    Some(
        "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d  mold-2.41.0-x86_64-linux.tar.gz  extra\n"
    ),
    "invalid SHA-256 checksum recorded for mold-2.41.0-x86_64-linux.tar.gz in"
)]
fn checksum_rows_fail_before_curl(
    #[case] contents: Option<&str>,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let path = "candidate/tools/mold/SHA256SUMS";
        match contents {
            Some(text) => scratch.write_relative(path, text.as_bytes())?,
            None => scratch.remove_relative(path)?,
        }
        let request = Request::new(Entry::Install);
        let separator = if diagnostic.ends_with(" in") {
            " "
        } else {
            ": "
        };
        let full_diagnostic = format!("{diagnostic}{separator}{}", scratch.path.join(path));
        let expected = oracle::expected(
            1,
            oracle::host_calls(scratch, &request),
            Some(full_diagnostic),
            oracle::prefix_state(false, false),
        );
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::curl(
    Failure::Curl,
    5,
    "failed to download mold-2.41.0-x86_64-linux.tar.gz",
    false
)]
#[case::sha(
    Failure::Checksum,
    6,
    "checksum mismatch for mold-2.41.0-x86_64-linux.tar.gz; refusing to install",
    false
)]
#[case::tar(
    Failure::Tar,
    7,
    "failed to unpack mold-2.41.0-x86_64-linux.tar.gz",
    false
)]
#[case::declared_rustup(
    Failure::DeclaredInstall,
    9,
    "failed to install toolchain 1.92.0 and its components",
    true
)]
#[case::stable_rustup(
    Failure::StableInstall,
    10,
    "failed to install stable toolchain 1.92.0 with rustfmt",
    true
)]
fn install_stage_failure_stops_and_cleans(
    #[case] failure: Failure,
    #[case] call_count: usize,
    #[case] diagnostic: &str,
    #[case] has_payload: bool,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.controls.failure = Some(failure);
        let install = oracle::install_expectation("x86_64", oracle::STABLE)?;
        let mut calls = oracle::install_calls(scratch, &request, &install);
        calls.truncate(call_count);
        calls
            .last_mut()
            .ok_or_else(|| "expected failing call is absent".to_owned())?
            .status = 97;
        let sha = if failure == Failure::Curl {
            Vec::new()
        } else {
            oracle::checksum_stdin(&install.digest, &install.archive)
        };
        let prefix_exists = failure != Failure::Curl && failure != Failure::Checksum;
        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(prefix_exists, has_payload),
        );
        expected.sha_stdin = sha;
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::missing(PayloadMode::Missing, None, None, "installed mold is unusable")]
#[case::unusable(
    PayloadMode::Installed,
    Some(Failure::InstalledMoldVersion),
    None,
    "installed mold is unusable"
)]
#[case::wrong_version(
    PayloadMode::Installed,
    None,
    Some("mold 0.0.0"),
    "installed mold version does not match 2.41.0"
)]
fn extracted_binary_is_checked_at_actual_prefix(
    #[case] payload: PayloadMode,
    #[case] failure: Option<Failure>,
    #[case] version: Option<&str>,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.controls.payload = payload;
        request.controls.failure = failure;
        request.controls.mold_version_output = version.map(str::to_owned);
        let install = oracle::install_expectation("x86_64", oracle::STABLE)?;
        let executable = payload == PayloadMode::Installed;
        let mut calls = oracle::install_calls(scratch, &request, &install);
        calls.truncate(if executable { 8 } else { 7 });
        if failure == Some(Failure::InstalledMoldVersion) {
            calls
                .last_mut()
                .ok_or_else(|| "prefix mold call is absent".to_owned())?
                .status = 97;
        }
        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(true, executable),
        );
        expected.sha_stdin = oracle::checksum_stdin(&install.digest, &install.archive);
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::empty_prefix(true)]
#[case::unrelated_tmp_marker_survives(false)]
fn prefix_and_temp_ownership(#[case] empty_prefix: bool) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        if !empty_prefix {
            oracle::successful_entry(scratch, &request)?;
            return Ok(());
        }
        request.controls.environment.push(EnvironmentChange {
            name: "BUILD_TOOLS_PREFIX".to_owned(),
            value: Some(String::new()),
        });
        let expected = oracle::expected(
            1,
            oracle::host_calls(scratch, &request),
            Some("BUILD_TOOLS_PREFIX must not be empty".to_owned()),
            oracle::prefix_state(false, false),
        );
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::seven_primary_rows(oracle::PRIMARY_ROWS)]
fn all_published_digest_rows_are_exact(#[case] expected_rows: &str) -> Result<(), String> {
    with_scratch(|scratch| {
        let actual = scratch.read_relative("candidate/tools/mold/SHA256SUMS")?;
        oracle::require(
            actual == expected_rows.as_bytes(),
            "published SHA256SUMS bytes",
            &format!("{expected_rows:?}"),
            &format!("{:?}", String::from_utf8_lossy(&actual)),
        )
    })
}

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
