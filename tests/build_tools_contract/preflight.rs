//! Independent preflight contracts for pinned tool availability and inputs.
//!
//! Each named case binds one planned C2 row to full, literal process evidence.

use rstest::rstest;

use super::{
    Entry, EnvironmentChange, Failure, Host, Request, install_oracles as oracle,
    preflight_oracles::{
        set_call_status, skip_component_query, verify_boot_failure, verify_host_failure,
        verify_install_then_check, verify_missing_requirements, verify_missing_rustup,
        verify_stable_discovery_failure,
    },
    support::with_scratch,
};

#[rstest]
#[case::install_missing_mold(Entry::Install, "tools/mold/VERSION", None, "missing version pin")]
#[case::install_missing_stable(
    Entry::Install,
    "tools/rust-stable/VERSION",
    None,
    "missing version pin"
)]
#[case::install_empty(Entry::Install, "tools/mold/VERSION", Some("\n"), "empty version pin")]
#[case::install_whitespace_only(
    Entry::Install,
    "tools/mold/VERSION",
    Some(" \t \n"),
    "empty version pin"
)]
#[case::install_multiline(
    Entry::Install,
    "tools/mold/VERSION",
    Some("2.41.0\n2.41.0\n"),
    "expected one line in version pin"
)]
#[case::install_embedded_whitespace(
    Entry::Install,
    "tools/mold/VERSION",
    Some("2.41. 0\n"),
    "version pin contains whitespace"
)]
#[case::install_non_version_mold(
    Entry::Install,
    "tools/mold/VERSION",
    Some("stable\n"),
    "invalid version pin"
)]
#[case::install_non_version_stable(
    Entry::Install,
    "tools/rust-stable/VERSION",
    Some("nightly\n"),
    "invalid version pin"
)]
#[case::check_missing_mold(Entry::Check, "tools/mold/VERSION", None, "missing version pin")]
#[case::check_missing_stable(
    Entry::Check,
    "tools/rust-stable/VERSION",
    None,
    "missing version pin"
)]
#[case::check_empty(Entry::Check, "tools/mold/VERSION", Some("\n"), "empty version pin")]
#[case::check_whitespace_only(
    Entry::Check,
    "tools/mold/VERSION",
    Some(" \t \n"),
    "empty version pin"
)]
#[case::check_multiline(
    Entry::Check,
    "tools/mold/VERSION",
    Some("2.41.0\n2.41.0\n"),
    "expected one line in version pin"
)]
#[case::check_embedded_whitespace(
    Entry::Check,
    "tools/mold/VERSION",
    Some("2.41. 0\n"),
    "version pin contains whitespace"
)]
#[case::check_non_version_mold(
    Entry::Check,
    "tools/mold/VERSION",
    Some("stable\n"),
    "invalid version pin"
)]
#[case::check_non_version_stable(
    Entry::Check,
    "tools/rust-stable/VERSION",
    Some("nightly\n"),
    "invalid version pin"
)]
fn pins_fail_before_side_effects(
    #[case] entry: Entry,
    #[case] path: &str,
    #[case] contents: Option<&str>,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let relative_path = format!("candidate/{path}");
        match contents {
            Some(text) => scratch.write_relative(&relative_path, text.as_bytes())?,
            None => scratch.remove_relative(&relative_path)?,
        }
        verify_boot_failure(scratch, entry, diagnostic)
    })
}

#[rstest]
#[case::missing_channel("[toolchain]\ncomponents=[]\n", "expected one valid channel")]
#[case::duplicate_channel(
    "[toolchain]\nchannel = \"1.92.0\"\nchannel=\"1.92.0\"\n",
    "expected one valid channel"
)]
#[case::unquoted_channel("[toolchain]\nchannel=1.92.0\n", "expected one valid channel")]
#[case::non_version_channel("[toolchain]\nchannel=\"stable\"\n", "expected one valid channel")]
#[case::duplicate_components(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[]\ncomponents=[]\n",
    "could not read components"
)]
#[case::stray_prefix(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[prefix \"rustfmt\"]\n",
    "could not read components"
)]
#[case::adjacent_strings(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\" \"clippy\"]\n",
    "could not read components"
)]
#[case::missing_comma(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\n\"rustfmt\"\n\"clippy\"\n]\n",
    "could not read components"
)]
#[case::double_comma(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\",,\"clippy\"]\n",
    "could not read components"
)]
#[case::invalid_component(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rust src\"]\n",
    "could not read components"
)]
#[case::unterminated_array(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\",\n",
    "could not read components"
)]
#[case::bad_close_tail(
    "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\"] garbage\n",
    "could not read components"
)]
#[case::duplicate_toolchain_table(
    "[toolchain]\nchannel = \"1.92.0\"\n[toolchain]\n",
    "expected one valid channel"
)]
#[case::later_header_missing_close(
    "[toolchain]\nchannel = \"1.92.0\"\n[toolchain\n",
    "expected one valid channel"
)]
#[case::later_header_trailing_garbage(
    "[toolchain]\nchannel = \"1.92.0\"\n[toolchain] trailing\n",
    "expected one valid channel"
)]
fn strict_toolchain_input_rejected(
    #[case] contents: &str,
    #[case] diagnostic: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        scratch.write_relative("candidate/rust-toolchain.toml", contents.as_bytes())?;
        verify_boot_failure(scratch, Entry::Check, diagnostic)
    })
}

#[rstest]
#[case::empty_array("[toolchain]\nchannel = \"1.92.0\"\ncomponents=[]\n", "1.92.0", &[], "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n")]
#[case::inline_duplicates("[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rust-analyzer\", \"rustfmt\", \"clippy\", \"rustfmt\"]\n", "1.92.0", &[], "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n")]
#[case::multiline_trailing_comma("[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\n\"rustfmt\",\n\"rust-src\",\n]\n", "1.92.0", &["rust-src"], "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\nrust-src (installed)\n")]
#[case::extra_and_llvm_alias("[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rust-src\", \"llvm-tools-preview\", \"rust-src\"]\n", "1.92.0", &["rust-src", "llvm-tools-preview"], "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\nrust-src (installed)\nllvm-tools (installed)\n")]
#[case::valid_unrelated_table_after_toolchain("[toolchain]\nchannel = \"1.92.0\"\ncomponents=[]\n[package.metadata]\nchannel=\"ignored\"\n[[targets.windows]]\nname=\"ignored\"\n", "1.92.0", &[], "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n")]
fn valid_components_are_deduplicated_and_preserved(
    #[case] contents: &str,
    #[case] channel: &str,
    #[case] extras: &[&str],
    #[case] component_output: &str,
) -> Result<(), String> {
    with_scratch(|scratch| {
        scratch.write_relative("candidate/rust-toolchain.toml", contents.as_bytes())?;
        let mut request = Request::new(Entry::Install);
        let [declared, _stable] = &mut request.controls.rustup.component_output;
        *declared = component_output.to_owned();
        verify_install_then_check(scratch, &mut request, channel, extras)
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
#[case::mold_absent("mold_absent", "mold not found on PATH")]
#[case::mold_unusable("mold_unusable", "is unusable or cannot report its version")]
#[case::mold_wrong_pin("mold_wrong_pin", "does not match the pin")]
#[case::clang_absent("clang_absent", "clang not found on PATH")]
#[case::declared_toolchain_absent("declared_toolchain_absent", "is not installed")]
#[case::declared_component_absent(
    "declared_component_absent",
    "is missing required component rust-analyzer"
)]
#[case::stable_toolchain_absent("stable_toolchain_absent", "is not installed")]
#[case::stable_rustfmt_absent("stable_rustfmt_absent", "is missing required component rustfmt")]
fn preflight_missing_requirements_fail(
    #[case] scenario: &str,
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
        set_call_status(&mut calls, "clang", arguments, 97)?;
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
        set_call_status(&mut calls, "rustup", arguments, 97)?;
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
