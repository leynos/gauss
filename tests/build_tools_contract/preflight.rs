//! Independent preflight contracts for pinned tool availability and inputs.
//!
//! Each named case binds one planned C2 row to full, literal process evidence.

use rstest::rstest;

use super::{
    Entry, EnvironmentChange, Failure, Host, Request, install_oracles as oracle,
    preflight_cases as cases,
    preflight_oracles::{
        CallOccurrence, InvocationSelection, MissingRequirement, set_call_status,
        skip_component_query, verify_boot_failure, verify_host_failure, verify_install_then_check,
        verify_missing_requirements, verify_missing_rustup, verify_stable_discovery_failure,
    },
    support::with_scratch,
};

#[rstest]
#[case::install_missing_mold(cases::INSTALL_MISSING_MOLD)]
#[case::install_missing_stable(cases::INSTALL_MISSING_STABLE)]
#[case::install_empty(cases::INSTALL_EMPTY)]
#[case::install_whitespace_only(cases::INSTALL_WHITESPACE_ONLY)]
#[case::install_multiline(cases::INSTALL_MULTILINE)]
#[case::install_embedded_whitespace(cases::INSTALL_EMBEDDED_WHITESPACE)]
#[case::install_non_version_mold(cases::INSTALL_NON_VERSION_MOLD)]
#[case::install_non_version_stable(cases::INSTALL_NON_VERSION_STABLE)]
#[case::check_missing_mold(cases::CHECK_MISSING_MOLD)]
#[case::check_missing_stable(cases::CHECK_MISSING_STABLE)]
#[case::check_empty(cases::CHECK_EMPTY)]
#[case::check_whitespace_only(cases::CHECK_WHITESPACE_ONLY)]
#[case::check_multiline(cases::CHECK_MULTILINE)]
#[case::check_embedded_whitespace(cases::CHECK_EMBEDDED_WHITESPACE)]
#[case::check_non_version_mold(cases::CHECK_NON_VERSION_MOLD)]
#[case::check_non_version_stable(cases::CHECK_NON_VERSION_STABLE)]
fn pins_fail_before_side_effects(#[case] case: cases::PinCase) -> Result<(), String> {
    with_scratch(|scratch| {
        let relative_path = format!("candidate/{}", case.path);
        match case.contents {
            Some(text) => scratch.write_relative(&relative_path, text.as_bytes())?,
            None => scratch.remove_relative(&relative_path)?,
        }
        verify_boot_failure(scratch, case.entry, case.diagnostic)
    })
}

#[rstest]
#[case::missing_channel(cases::MISSING_CHANNEL)]
#[case::duplicate_channel(cases::DUPLICATE_CHANNEL)]
#[case::unquoted_channel(cases::UNQUOTED_CHANNEL)]
#[case::non_version_channel(cases::NON_VERSION_CHANNEL)]
#[case::duplicate_components(cases::DUPLICATE_COMPONENTS)]
#[case::stray_prefix(cases::STRAY_PREFIX)]
#[case::adjacent_strings(cases::ADJACENT_STRINGS)]
#[case::missing_comma(cases::MISSING_COMMA)]
#[case::double_comma(cases::DOUBLE_COMMA)]
#[case::invalid_component(cases::INVALID_COMPONENT)]
#[case::unterminated_array(cases::UNTERMINATED_ARRAY)]
#[case::bad_close_tail(cases::BAD_CLOSE_TAIL)]
#[case::duplicate_toolchain_table(cases::DUPLICATE_TOOLCHAIN_TABLE)]
#[case::later_header_missing_close(cases::LATER_HEADER_MISSING_CLOSE)]
#[case::later_header_trailing_garbage(cases::LATER_HEADER_TRAILING_GARBAGE)]
fn strict_toolchain_input_rejected(#[case] case: cases::ToolchainRejection) -> Result<(), String> {
    with_scratch(|scratch| {
        scratch.write_relative("candidate/rust-toolchain.toml", case.contents.as_bytes())?;
        verify_boot_failure(scratch, Entry::Check, case.diagnostic)
    })
}

#[rstest]
#[case::empty_array(cases::EMPTY_ARRAY)]
#[case::inline_duplicates(cases::INLINE_DUPLICATES)]
#[case::multiline_trailing_comma(cases::MULTILINE_TRAILING_COMMA)]
#[case::extra_and_llvm_alias(cases::EXTRA_AND_LLVM_ALIAS)]
#[case::valid_unrelated_table_after_toolchain(cases::VALID_UNRELATED_TABLE_AFTER_TOOLCHAIN)]
fn valid_components_are_deduplicated_and_preserved(
    #[case] case: cases::InstalledToolchainCase,
) -> Result<(), String> {
    with_scratch(|scratch| {
        scratch.write_relative("candidate/rust-toolchain.toml", case.contents.as_bytes())?;
        let mut request = Request::new(Entry::Install);
        let [declared, _stable] = &mut request.controls.rustup.component_output;
        *declared = case.component_output.to_owned();
        verify_install_then_check(scratch, &mut request, case.channel, case.extras)
    })
}

#[rstest]
#[case::empty("")]
#[case::nonempty("-D\u{1f}warnings")]
fn encoded_flags_presence_is_rejected(#[case] value: &str) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.controls.environment.push(EnvironmentChange {
            name: "CARGO_ENCODED_RUSTFLAGS".to_owned(),
            value: Some(value.to_owned()),
        });
        verify_host_failure(scratch, &request, "CARGO_ENCODED_RUSTFLAGS is set")
    })
}

#[rstest]
#[case::explicit_riscv_gnu(Some("riscv64gc-unknown-linux-gnu"), None)]
#[case::cargo_s390_musl(None, Some("s390x-unknown-linux-musl"))]
#[case::explicit_custom_linux(Some("vendor-custom-linux-special"), None)]
#[case::explicit_overrides_cargo(Some("x86_64-unknown-linux-gnu"), Some("aarch64-apple-darwin"))]
fn non_linux_rejects_every_linux_target(
    #[case] build_target: Option<&str>,
    #[case] cargo_target: Option<&str>,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.host = Host::Darwin;
        for (name, maybe_target) in [
            ("BUILD_TOOLS_TARGET", build_target),
            ("CARGO_BUILD_TARGET", cargo_target),
        ] {
            if let Some(selected_target) = maybe_target {
                request.controls.environment.push(EnvironmentChange {
                    name: name.to_owned(),
                    value: Some(selected_target.to_owned()),
                });
            }
        }
        verify_host_failure(scratch, &request, "Linux target requires a Linux host")
    })
}

#[rstest]
#[case::linux(Host::Linux)]
#[case::darwin_native(Host::Darwin)]
#[case::freebsd_native(Host::FreeBsd)]
fn valid_preflight_checks_expected_tools(#[case] host: Host) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.host = host;
        oracle::successful_entry(scratch, &request)?;
        Ok(())
    })
}

#[rstest]
#[case::mold_absent(MissingRequirement::MoldAbsent, "mold not found on PATH")]
#[case::mold_unusable(
    MissingRequirement::MoldUnusable,
    "is unusable or cannot report its version"
)]
#[case::mold_wrong_pin(MissingRequirement::MoldWrongPin, "does not match the pin")]
#[case::clang_absent(MissingRequirement::ClangAbsent, "clang not found on PATH")]
#[case::declared_toolchain_absent(MissingRequirement::DeclaredToolchainAbsent, "is not installed")]
#[case::declared_component_absent(
    MissingRequirement::DeclaredComponentAbsent,
    "is missing required component rust-analyzer"
)]
#[case::stable_toolchain_absent(MissingRequirement::StableToolchainAbsent, "is not installed")]
#[case::stable_rustfmt_absent(
    MissingRequirement::StableRustfmtAbsent,
    "is missing required component rustfmt"
)]
fn preflight_missing_requirements_fail(
    #[case] scenario: MissingRequirement,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    verify_missing_requirements(scenario, diagnostic)
}

#[rstest]
#[case::version(Failure::ClangVersion, &["--version"], false, "cannot report its version")]
#[case::syntax(Failure::ClangSyntax, &["-x", "c", "-fsyntax-only", "-"], true, "cannot compile a basic C translation unit")]
fn clang_failure_is_observable(
    #[case] failure: Failure,
    #[case] arguments: &[&str],
    #[case] syntax_reached: bool,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.controls.failure = Some(failure);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        calls.retain(|call| {
            call.tool != "clang"
                || syntax_reached
                || call
                    .argv
                    .iter()
                    .map(String::as_str)
                    .eq(arguments.iter().copied())
        });
        set_call_status(
            &mut calls,
            &InvocationSelection {
                tool: "clang",
                arguments,
                occurrence: CallOccurrence::First,
            },
            97,
        )?;
        let mut expected = oracle::expected(
            1,
            calls,
            Some(diagnostic.to_owned()),
            oracle::prefix_state(false, false),
        );
        if syntax_reached {
            expected.clang_stdin = oracle::CLANG_SOURCE.to_vec();
        }
        oracle::verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}

#[rstest]
#[case::toolchain_list(Failure::DeclaredToolchainList, &["toolchain", "list"], true, "could not list installed Rust toolchains")]
#[case::component_list(Failure::DeclaredComponentList, &["component", "list", "--toolchain", oracle::STABLE, "--installed"], false, "could not list components for toolchain")]
fn rustup_discovery_failure_propagates(
    #[case] failure: Failure,
    #[case] arguments: &[&str],
    #[case] skip_declared_component: bool,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Check);
        request.controls.failure = Some(failure);
        let mut calls = oracle::check_calls(scratch, &request, oracle::STABLE);
        if skip_declared_component {
            skip_component_query(&mut calls, 1)?;
        }
        set_call_status(
            &mut calls,
            &InvocationSelection {
                tool: "rustup",
                arguments,
                occurrence: CallOccurrence::First,
            },
            97,
        )?;
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

#[rstest]
#[case::nightly_2026_05_28("nightly-2026-05-28")]
fn dated_channel_remains_pinned(#[case] channel: &str) -> Result<(), String> {
    with_scratch(|scratch| {
        let contents = format!("[toolchain]\nchannel=\"{channel}\"\ncomponents=[]\n");
        scratch.write_relative("candidate/rust-toolchain.toml", contents.as_bytes())?;
        let mut request = Request::new(Entry::Install);
        let [declared, _stable] = &mut request.controls.rustup.list_output;
        *declared = format!("{channel}\n");
        verify_install_then_check(scratch, &mut request, channel, &[])
    })
}

#[rstest]
#[case::toolchain_list(
    Failure::StableToolchainList,
    &["toolchain", "list"],
    true,
    "could not list installed Rust toolchains",
)]
#[case::component_list(
    Failure::StableComponentList,
    &["component", "list", "--toolchain", oracle::STABLE, "--installed"],
    false,
    "could not list components for toolchain",
)]
fn stable_rustup_discovery_failure_propagates(
    #[case] failure: Failure,
    #[case] arguments: &[&str],
    #[case] skip_stable_component: bool,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    verify_stable_discovery_failure(failure, arguments, skip_stable_component, diagnostic)
}

#[test]
fn missing_rustup_reports_both_toolchain_checks() -> Result<(), String> {
    verify_missing_rustup()
}
