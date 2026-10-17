//! Child launch, probes, and explicit process-environment controls.
//!
//! This module is the only subprocess boundary; all paths and fixture values
//! come from the capability-owned Scratch and the typed request.

use std::process::Command;

use super::{
    Controls, Failure, Host, InjectedFunction, InjectionMode, PayloadMode, Probe, ProbeRequest,
    Request, RustupFixture,
    model::{ChildFixture, RawOutput},
    process_inputs::apply_environment_changes,
    support::Scratch,
};

/// Starts exactly one production entrypoint in privileged Bash.
pub(super) fn run_entry(scratch: &Scratch, request: &Request) -> Result<RawOutput, String> {
    let functions = match request.controls.injection {
        InjectionMode::EntryFunctions => super::ENTRY_FUNCTIONS,
        InjectionMode::None | InjectionMode::StartupFiles => &[],
    };
    let fixture = ChildFixture {
        host: request.host,
        machine: &request.machine,
        controls: &request.controls,
        functions,
        startup_files: request.controls.injection == InjectionMode::StartupFiles,
    };
    let script = scratch.path.join("candidate").join(request.entry.script());
    let mut command = Command::new("/bin/bash");
    command.args(["--noprofile", "--norc", "-p"]);
    command.arg(script.as_str());
    if request.controls.unexpected_argument {
        command.arg("unexpected");
    }
    apply_child_environment(&mut command, scratch, &fixture)?;
    capture(command, "production entrypoint")
}

/// Runs an import control or directly executes one privileged fake.
pub(super) fn run_probe(scratch: &Scratch, request: &ProbeRequest) -> Result<RawOutput, String> {
    let controls = Controls::default();
    let functions = probe_functions(request)?;
    let fixture = ChildFixture {
        host: Host::Linux,
        machine: "x86_64",
        controls: &controls,
        functions: &functions,
        startup_files: false,
    };
    let mut command = probe_command(scratch, request);
    apply_child_environment(&mut command, scratch, &fixture)?;
    capture(command, "privileged Bash isolation probe")
}

fn probe_functions(request: &ProbeRequest) -> Result<Vec<InjectedFunction>, String> {
    let required = match request.probe {
        Probe::CurlImportControl => vec![InjectedFunction::Curl],
        Probe::PrintfImportControl => vec![InjectedFunction::Printf],
        Probe::FakeCurl | Probe::InstalledMold => request.functions.clone(),
    };
    ensure_unique_functions(&required)?;
    Ok(required)
}

fn probe_command(scratch: &Scratch, request: &ProbeRequest) -> Command {
    match request.probe {
        Probe::CurlImportControl => {
            let mut command = Command::new("/bin/bash");
            command.args(["--noprofile", "--norc", "-c", "curl"]);
            command
        }
        Probe::PrintfImportControl => {
            let mut command = Command::new("/bin/bash");
            command.args(["--noprofile", "--norc", "-c", "printf control"]);
            command
        }
        Probe::FakeCurl => {
            let mut command = Command::new(scratch.path.join("bin/curl").as_str());
            command.args([
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
            ]);
            command.arg(scratch.path.join("probe-archive").as_str());
            command.arg("https://example.invalid/fixture.tar.gz");
            command
        }
        Probe::InstalledMold => {
            let mut command = Command::new(scratch.path.join("prefix/bin/mold").as_str());
            command.arg("--version");
            command
        }
    }
}

fn apply_child_environment(
    command: &mut Command,
    scratch: &Scratch,
    fixture: &ChildFixture<'_>,
) -> Result<(), String> {
    remove_known_inputs(command);
    command
        .env("PATH", scratch.path.join("bin").as_str())
        .env("HOME", scratch.path.join("home").as_str())
        .env("TMPDIR", scratch.path.join("tmp").as_str())
        .env("BUILD_TOOLS_PREFIX", scratch.path.join("prefix").as_str());
    apply_owned_paths(command, scratch);
    apply_fake_controls(command, scratch, fixture)?;
    apply_environment_changes(command, scratch, &fixture.controls.environment)?;
    inject_functions(command, fixture.functions);
    if fixture.startup_files {
        let startup = scratch.path.join("fake/startup.sh");
        command
            .env("BASH_ENV", startup.as_str())
            .env("ENV", startup.as_str());
    }
    Ok(())
}

fn remove_known_inputs(command: &mut Command) {
    for name in [
        "BASH_ENV",
        "ENV",
        "MOLD_VERSION_FILE",
        "MOLD_SHA256SUMS_FILE",
        "RUST_TOOLCHAIN_FILE",
        "BUILD_TOOLS_PREFIX",
        "BUILD_TOOLS_TARGET",
        "CARGO_BUILD_TARGET",
        "CARGO_ENCODED_RUSTFLAGS",
        "BUILD_TOOLS_MOLD_REQUIRED",
        "BASH_FUNC_curl%%",
        "BASH_FUNC_uname%%",
        "BASH_FUNC_printf%%",
        "BASH_FUNC_command%%",
        "BASH_FUNC_mold%%",
    ] {
        command.env_remove(name);
    }
}

fn apply_owned_paths(command: &mut Command, scratch: &Scratch) {
    command
        .env("GAUSS_CALL_LOG", scratch.path.join("calls.log").as_str())
        .env("GAUSS_SHA_STDIN", scratch.path.join("sha.stdin").as_str())
        .env(
            "GAUSS_CLANG_STDIN",
            scratch.path.join("clang.stdin").as_str(),
        )
        .env("GAUSS_SHADOW_LOG", scratch.path.join("shadow.log").as_str())
        .env(
            "GAUSS_FAKE_COMMON",
            scratch.path.join(super::FAKE_COMMON_FILE).as_str(),
        )
        .env(
            "GAUSS_MOLD_PAYLOAD",
            scratch.path.join(super::FAKE_MOLD_PAYLOAD_FILE).as_str(),
        )
        .env("GAUSS_PREFIX", scratch.path.join("prefix").as_str())
        .env(
            "GAUSS_RUSTUP_INSTALL_COUNT",
            scratch.path.join("state/rustup-install-count").as_str(),
        )
        .env(
            "GAUSS_RUSTUP_LIST_COUNT",
            scratch.path.join("state/rustup-list-count").as_str(),
        )
        .env(
            "GAUSS_RUSTUP_COMPONENT_COUNT",
            scratch.path.join("state/rustup-component-count").as_str(),
        );
}

fn apply_fake_controls(
    command: &mut Command,
    scratch: &Scratch,
    fixture: &ChildFixture<'_>,
) -> Result<(), String> {
    command
        .env("GAUSS_HOST_SYSTEM", fixture.host.system())
        .env("GAUSS_HOST_MACHINE", fixture.machine)
        .env(
            "GAUSS_FAILURE_STAGE",
            failure_stage(fixture.controls.failure),
        )
        .env(
            "GAUSS_MOLD_PAYLOAD_MODE",
            match fixture.controls.payload {
                PayloadMode::Installed => "installed",
                PayloadMode::Missing => "missing",
            },
        );
    command
        .env(
            "GAUSS_MOLD_VERSION_OUTPUT",
            fixture
                .controls
                .mold_version_output
                .as_deref()
                .unwrap_or("mold 2.41.0"),
        )
        .env(
            "GAUSS_CLANG_VERSION_OUTPUT",
            fixture
                .controls
                .clang_version_output
                .as_deref()
                .unwrap_or("clang fixture"),
        );
    configure_rustup(command, &fixture.controls.rustup, fixture.controls.failure)?;
    validate_scratch_paths(scratch)?;
    Ok(())
}

fn configure_rustup(
    command: &mut Command,
    fixture: &RustupFixture,
    failure: Option<Failure>,
) -> Result<(), String> {
    let [mut declared_install, mut stable_install] = fixture.install_status;
    let [mut declared_list, mut stable_list] = fixture.list_status;
    let [mut declared_components, mut stable_components] = fixture.component_status;
    match failure {
        Some(Failure::DeclaredInstall) => declared_install = 97,
        Some(Failure::StableInstall) => stable_install = 97,
        Some(Failure::DeclaredToolchainList) => declared_list = 97,
        Some(Failure::StableToolchainList) => stable_list = 97,
        Some(Failure::DeclaredComponentList) => declared_components = 97,
        Some(Failure::StableComponentList) => stable_components = 97,
        _ => {}
    }
    for (ordinal, status) in [(1, declared_install), (2, stable_install)] {
        command.env(
            format!("GAUSS_RUSTUP_INSTALL_STATUS_{ordinal}"),
            exit_value(status)?,
        );
    }
    let [declared_list_output, stable_list_output] = &fixture.list_output;
    for (ordinal, status, output) in [
        (1, declared_list, declared_list_output),
        (2, stable_list, stable_list_output),
    ] {
        command.env(
            format!("GAUSS_RUSTUP_LIST_STATUS_{ordinal}"),
            exit_value(status)?,
        );
        command.env(format!("GAUSS_RUSTUP_LIST_OUTPUT_{ordinal}"), output);
    }
    let [declared_component_output, stable_component_output] = &fixture.component_output;
    for (ordinal, status, output) in [
        (1, declared_components, declared_component_output),
        (2, stable_components, stable_component_output),
    ] {
        command.env(
            format!("GAUSS_RUSTUP_COMPONENT_STATUS_{ordinal}"),
            exit_value(status)?,
        );
        command.env(format!("GAUSS_RUSTUP_COMPONENT_OUTPUT_{ordinal}"), output);
    }
    Ok(())
}

fn validate_scratch_paths(scratch: &Scratch) -> Result<(), String> {
    let root = scratch.path.as_str();
    for path in [
        scratch.path.join("candidate"),
        scratch.path.join("bin"),
        scratch.path.join("tmp"),
        scratch.path.join("prefix"),
    ] {
        if path.strip_prefix(root).is_err() || path.as_str().split('/').any(|part| part == "..") {
            return Err("controlled path escaped its owned scratch root".to_owned());
        }
    }
    Ok(())
}

fn inject_functions(command: &mut Command, functions: &[InjectedFunction]) {
    for function in functions {
        let name = function.name();
        let variable = format!("BASH_FUNC_{name}%%");
        let body = format!(
            "() {{ builtin printf \"%s\\n\" {name} >> \"$GAUSS_SHADOW_LOG\"; return 97; }}"
        );
        command.env(variable, body);
    }
}

fn ensure_unique_functions(functions: &[InjectedFunction]) -> Result<(), String> {
    let mut names = Vec::new();
    for function in functions {
        if names.contains(&function.name()) {
            return Err(format!(
                "duplicate injected Bash function: {}",
                function.name()
            ));
        }
        names.push(function.name());
    }
    Ok(())
}

const fn failure_stage(failure: Option<Failure>) -> &'static str {
    match failure {
        Some(Failure::UnameSystem) => "uname-system",
        Some(Failure::UnameMachine) => "uname-machine",
        Some(Failure::Curl) => "curl",
        Some(Failure::Checksum) => "checksum",
        Some(Failure::Tar) => "tar",
        Some(Failure::MoldVersion) => "mold-version",
        Some(Failure::InstalledMoldVersion) => "installed-mold-version",
        Some(Failure::ClangVersion) => "clang-version",
        Some(Failure::ClangSyntax) => "clang-syntax",
        _ => "",
    }
}

fn exit_value(status: i32) -> Result<String, String> {
    if !(0..=255).contains(&status) {
        return Err(format!("fake exit status is outside 0..=255: {status}"));
    }
    Ok(status.to_string())
}

fn capture(mut command: Command, context: &str) -> Result<RawOutput, String> {
    let output = command
        .output()
        .map_err(|error| format!("starting {context}: {error}"))?;
    Ok(RawOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}
