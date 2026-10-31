//! Independent expected calls and process observations for C1 cases.
//!
//! These builders use literal release rows and controlled request values, never
//! output-derived expectations from the production scripts.

use super::{
    Entry, EnvironmentChange, Host, Invocation, Observation, Program, Request,
    support::{Scratch, with_scratch},
};

pub(super) use super::observation_oracles::{
    ExpectedObservation, compare_clean_injected, expected, prefix_state, require,
    verify_observation,
};

/// Pinned mold release version used in independent archive expectations.
const MOLD: &str = "2.41.0";
/// Pinned stable Rust toolchain used by install and preflight expectations.
pub(super) const STABLE: &str = "1.92.0";
/// Harmless archive payload written by the direct fake-curl probe.
pub(super) const INERT_ARCHIVE: &[u8] = b"inert archive fixture\n";
/// Minimal C translation unit expected on Linux preflight.
pub(super) const CLANG_SOURCE: &[u8] = b"int main(void) { return 0; }\n";
/// Exact seven published primary mold checksum rows, independent of the candidate copy.
pub(super) const PRIMARY_ROWS: &str = concat!(
    "946de2774b06a71346bd59b55fddba610b65b8d93c3a4a1559cc84e103472710  mold-2.41.0-aarch64-linux.tar.gz\n",
    "19c877255b52c2d8060ca1229fe139d18ab31a1e5614079b853d3be16bdda647  mold-2.41.0-arm-linux.tar.gz\n",
    "16b71ee699ffd6b3dc77bf28f954b50107ce6f16618e6b0725c31967af9f3445  mold-2.41.0-loongarch64-linux.tar.gz\n",
    "b1a045d9dcbdb1af523518beca9da38f3ee0f208964851831a0664cafaf9bc70  mold-2.41.0-ppc64le-linux.tar.gz\n",
    "9e04298a0444a83fb9864ee8520e2730fb84e89c5617478b87396888a753fa41  mold-2.41.0-riscv64-linux.tar.gz\n",
    "e5e42a2685967ad209d7568e29a0b2ea86ec9d462cbf5d4aa7398a8087ee1a9d  mold-2.41.0-s390x-linux.tar.gz\n",
    "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d  mold-2.41.0-x86_64-linux.tar.gz\n",
);

/// Inputs that independently describe one successful installer trace.
pub(super) struct InstallExpectation<'a> {
    /// Declared Rust channel expected by the installer.
    pub(super) channel: &'a str,
    /// Published archive basename expected for this Linux machine.
    pub(super) archive: String,
    /// Independently pinned SHA-256 digest for the archive.
    pub(super) digest: String,
    /// Additional declared components appended after the mandatory set.
    pub(super) extras: &'a [&'a str],
}

/// Creates one exact invocation from independently specified arguments.
pub(super) fn call(tool: &str, argv: &[&str], status: i32) -> Invocation {
    Invocation {
        tool: tool.to_owned(),
        argv: argv.iter().map(|argument| (*argument).to_owned()).collect(),
        status,
    }
}

/// Describes the two initial dirname calls for one entrypoint.
pub(super) fn boot(scratch: &Scratch, entry: Entry) -> Vec<Invocation> {
    let script = scratch.path.join("candidate").join(entry.script());
    let common = scratch.path.join("candidate/scripts/build-tools-common.sh");
    vec![
        call("dirname", &["--", script.as_str()], 0),
        call("dirname", &["--", common.as_str()], 0),
    ]
}

/// Describes successful host detection, including the Linux machine query.
pub(super) fn host_calls(scratch: &Scratch, request: &Request) -> Vec<Invocation> {
    let mut calls = boot(scratch, request.entry);
    calls.push(call("uname", &["-s"], 0));
    if request.host == Host::Linux {
        calls.push(call("uname", &["-m"], 0));
    }
    calls
}

/// Describes every successful install call in production order.
pub(super) fn install_calls(
    scratch: &Scratch,
    request: &Request,
    expected: &InstallExpectation<'_>,
) -> Vec<Invocation> {
    let mut calls = host_calls(scratch, request);
    if request.host == Host::Linux {
        let archive_path = format!("<OWNED_DOWNLOAD_DIR>/{}", expected.archive);
        let url = format!(
            "https://github.com/rui314/mold/releases/download/v{MOLD}/{}",
            expected.archive
        );
        calls.push(call(
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
                archive_path.as_str(),
                url.as_str(),
            ],
            0,
        ));
        calls.push(call("sha256sum", &["--check", "--status"], 0));
        let archive = format!("<OWNED_DOWNLOAD_DIR>/{}", expected.archive);
        let prefix = configured_prefix(scratch, request);
        calls.push(call(
            "tar",
            &[
                "--extract",
                "--gzip",
                "--strip-components=1",
                "--directory",
                prefix.as_str(),
                "--file",
                archive.as_str(),
            ],
            0,
        ));
        calls.push(call("prefix-mold", &["--version"], 0));
    }
    let mut components = vec!["rustfmt", "clippy", "rust-analyzer"];
    for extra in expected.extras {
        if !components.contains(extra) {
            components.push(*extra);
        }
    }
    calls.push(rustup_install(expected.channel, &components));
    calls.push(rustup_install(STABLE, &["rustfmt"]));
    calls
}

/// Describes successful preflight calls without deriving expectations from output.
pub(super) fn check_calls(scratch: &Scratch, request: &Request, channel: &str) -> Vec<Invocation> {
    let mut calls = host_calls(scratch, request);
    if request.host == Host::Linux {
        calls.push(call("mold", &["--version"], 0));
        calls.push(call("clang", &["--version"], 0));
        calls.push(call("clang", &["-x", "c", "-fsyntax-only", "-"], 0));
    }
    for toolchain in [channel, STABLE] {
        calls.push(call("rustup", &["toolchain", "list"], 0));
        calls.push(call(
            "rustup",
            &["component", "list", "--toolchain", toolchain, "--installed"],
            0,
        ));
    }
    calls
}

/// Builds the ordered minimal-profile rustup install arguments.
fn rustup_install(channel: &str, components: &[&str]) -> Invocation {
    let mut argv = vec![
        "toolchain".to_owned(),
        "install".to_owned(),
        channel.to_owned(),
        "--profile".to_owned(),
        "minimal".to_owned(),
    ];
    for component in components {
        argv.extend(["--component".to_owned(), (*component).to_owned()]);
    }
    owned_call("rustup", argv, 0)
}

/// Stores already-owned argv without changing argument order.
fn owned_call(tool: &str, argv: Vec<String>, status: i32) -> Invocation {
    Invocation {
        tool: tool.to_owned(),
        argv,
        status,
    }
}

/// Resolves only the explicit child prefix override or owned default.
fn configured_prefix(scratch: &Scratch, request: &Request) -> String {
    match request
        .controls
        .environment
        .iter()
        .find(|change| change.name == "BUILD_TOOLS_PREFIX")
    {
        Some(EnvironmentChange {
            value: Some(path), ..
        }) => path.clone(),
        Some(EnvironmentChange { value: None, .. }) => scratch.path.join("home/.local").to_string(),
        None => scratch.path.join("prefix").to_string(),
    }
}

/// Selects the canonical asset architecture for one accepted uname value.
fn asset_architecture(machine: &str) -> Result<&'static str, String> {
    match machine {
        "x86_64" | "amd64" => Ok("x86_64"),
        "aarch64" | "arm64" => Ok("aarch64"),
        "armv7l" | "armv8l" => Ok("arm"),
        "riscv64" => Ok("riscv64"),
        "ppc64le" => Ok("ppc64le"),
        "s390x" => Ok("s390x"),
        "loongarch64" => Ok("loongarch64"),
        _ => Err(format!("no independent release row for machine {machine}")),
    }
}

/// Reads the exact pinned digest from the independent seven-row fixture.
fn published_row(machine: &str) -> Result<(String, String), String> {
    let architecture = asset_architecture(machine)?;
    let archive = format!("mold-{MOLD}-{architecture}-linux.tar.gz");
    let row = PRIMARY_ROWS
        .lines()
        .find(|row| row.ends_with(archive.as_str()))
        .ok_or_else(|| format!("missing pinned mold row for {archive}"))?;
    let (digest, row_archive) = row
        .split_once("  ")
        .ok_or_else(|| format!("pinned mold row is malformed for {archive}"))?;
    if row_archive != archive.as_str() {
        return Err(format!(
            "pinned mold row names {row_archive}, expected {archive}"
        ));
    }
    Ok((archive, digest.to_owned()))
}

/// Formats checksum stdin with the sole permitted owned-root placeholder.
pub(super) fn checksum_stdin(digest: &str, archive: &str) -> Vec<u8> {
    format!("{digest}  <OWNED_DOWNLOAD_DIR>/{archive}\n").into_bytes()
}

/// Builds the expected release row for one declared channel.
pub(super) fn install_expectation<'a>(
    machine: &str,
    channel: &'a str,
) -> Result<InstallExpectation<'a>, String> {
    let (archive, digest) = published_row(machine)?;
    Ok(InstallExpectation {
        channel,
        archive,
        digest,
        extras: &[],
    })
}

/// Verifies a successful install or check entrypoint with independent evidence.
pub(super) fn successful_entry(
    scratch: &Scratch,
    request: &Request,
) -> Result<Observation, String> {
    let observation = scratch.run(request)?;
    verify_observation(
        scratch,
        &observation,
        successful_expectation(scratch, request)?,
    )?;
    Ok(observation)
}

/// Builds successful process expectations from the selected entrypoint.
fn successful_expectation(
    scratch: &Scratch,
    request: &Request,
) -> Result<ExpectedObservation, String> {
    match request.entry {
        Entry::Install => successful_install_expectation(scratch, request),
        Entry::Check => Ok(successful_check_expectation(scratch, request)),
    }
}

/// Builds the successful installer expectation with independent release evidence.
fn successful_install_expectation(
    scratch: &Scratch,
    request: &Request,
) -> Result<ExpectedObservation, String> {
    let install = install_expectation(&request.machine, STABLE)?;
    let linux = request.host == Host::Linux;
    let mut expected = expected(
        0,
        install_calls(scratch, request, &install),
        Some(success_diagnostic(request.entry, request.host).to_owned()),
        prefix_state(linux, linux),
    );
    if linux {
        expected.sha_stdin = checksum_stdin(&install.digest, &install.archive);
    }
    Ok(expected)
}

/// Builds the successful preflight expectation for the selected host.
fn successful_check_expectation(scratch: &Scratch, request: &Request) -> ExpectedObservation {
    let linux = request.host == Host::Linux;
    let mut expected = expected(
        0,
        check_calls(scratch, request, STABLE),
        Some(success_diagnostic(request.entry, request.host).to_owned()),
        prefix_state(false, false),
    );
    if linux {
        expected.clang_stdin = CLANG_SOURCE.to_vec();
    }
    expected
}

/// Selects the existing success diagnostic for one entrypoint and host.
const fn success_diagnostic(entry: Entry, host: Host) -> &'static str {
    match (entry, host) {
        (Entry::Install, Host::Linux) => "build-tools: ready; verify with:",
        (Entry::Install, Host::Darwin | Host::FreeBsd) => "mold is Linux-only; skipping",
        (Entry::Check, Host::Linux) => "build-tools: clang is usable at",
        (Entry::Check, Host::Darwin | Host::FreeBsd) => {
            "toolchain 1.92.0 and its required components are available"
        }
    }
}

/// Verifies one missing installer prerequisite at its production failure stage.
pub(super) fn verify_missing_installer_tool(
    unavailable: Program,
    after_mold: bool,
) -> Result<(), String> {
    with_scratch(|scratch| {
        let mut request = Request::new(Entry::Install);
        request.controls.unavailable.push(unavailable);

        let installation = install_expectation("x86_64", STABLE)?;
        let mut calls = if after_mold {
            install_calls(scratch, &request, &installation)
        } else {
            host_calls(scratch, &request)
        };
        calls.truncate(if after_mold { 8 } else { 4 });

        let diagnostic = format!(
            "{} not found on PATH; install it before running this script",
            unavailable.name()
        );
        let mut expected = expected(
            1,
            calls,
            Some(diagnostic),
            prefix_state(after_mold, after_mold),
        );
        if after_mold {
            expected.sha_stdin = checksum_stdin(&installation.digest, &installation.archive);
        }
        verify_observation(scratch, &scratch.run(&request)?, expected)
    })
}
