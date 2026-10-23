//! Typed requests, fixtures, and observations for build-tool contract cases.
//!
//! These models keep case intent separate from child launching, capability-owned
//! filesystem work, and strict process-record decoding.

use std::process::ExitStatus;

/// Production entrypoint exercised by a controlled request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Entry {
    /// Selects the production installer script for this child run.
    Install,
    /// Selects the production preflight script for this child run.
    Check,
}

impl Entry {
    pub(super) const fn script(self) -> &'static str {
        match self {
            Self::Install => "scripts/install-build-tools.sh",
            Self::Check => "scripts/check-build-tools.sh",
        }
    }
}

/// Synthetic operating system returned by the recorded uname fake.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Host {
    /// Makes the fake uname report Linux and a controlled machine name.
    Linux,
    /// Makes the fake uname report Darwin without Linux-only tools.
    Darwin,
    /// Makes the fake uname report FreeBSD without Linux-only tools.
    FreeBsd,
}

impl Host {
    pub(super) const fn system(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Darwin => "Darwin",
            Self::FreeBsd => "FreeBSD",
        }
    }
}

/// One controlled production stage that the fake process can fail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Failure {
    /// Fails the checked uname system-name query.
    UnameSystem,
    /// Fails the checked Linux machine-name query.
    UnameMachine,
    /// Fails the fake archive download.
    Curl,
    /// Fails the fake checksum verification stage.
    Checksum,
    /// Fails the fake archive extraction stage.
    Tar,
    /// Fails rustup install call one for the declared channel.
    DeclaredInstall,
    /// Fails rustup install call two for the stable channel.
    StableInstall,
    /// Fails toolchain discovery call one for the declared channel.
    DeclaredToolchainList,
    /// Fails toolchain discovery call two for the stable channel.
    StableToolchainList,
    /// Fails component discovery call one for the declared channel.
    DeclaredComponentList,
    /// Fails component discovery call two for the stable channel.
    StableComponentList,
    /// Fails the PATH-resolved mold version check.
    MoldVersion,
    /// Fails the extracted prefix mold version check.
    InstalledMoldVersion,
    /// Fails the fake clang version query.
    ClangVersion,
    /// Fails the fake clang syntax-check invocation.
    ClangSyntax,
}

/// Fake executable that may be hidden to represent an unavailable tool.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Program {
    /// Hides the PATH fake mold to model a missing executable.
    Mold,
    /// Hides the fake clang to model a missing executable.
    Clang,
    /// Hides fake rustup to model missing toolchain management.
    Rustup,
    /// Hides fake curl to model an unavailable downloader.
    Curl,
    /// Hides fake sha256sum to model missing checksum support.
    Sha256sum,
    /// Hides fake tar to model missing extraction support.
    Tar,
}

impl Program {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Mold => "mold",
            Self::Clang => "clang",
            Self::Rustup => "rustup",
            Self::Curl => "curl",
            Self::Sha256sum => "sha256sum",
            Self::Tar => "tar",
        }
    }
}

/// One isolation probe against a controlled Bash function or fake executable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Probe {
    /// Runs an unprivileged control proving curl-function import.
    CurlImportControl,
    /// Runs an unprivileged control proving printf-function import.
    PrintfImportControl,
    /// Runs the owned curl executable with selected function injections.
    FakeCurl,
    /// Runs the extracted mold payload with selected injections.
    InstalledMold,
}

/// Grouped function fixtures for one owned-path isolation probe.
pub(super) struct ProbeRequest {
    /// Chooses the fixed executable or import-control probe to run.
    pub(super) probe: Probe,
    /// Lists synthetic functions supplied to probes that accept them.
    pub(super) functions: Vec<InjectedFunction>,
}

/// Grouped controls for one entrypoint run.
pub(super) struct Controls {
    /// Selects one fake stage whose next matching call returns failure.
    pub(super) failure: Option<Failure>,
    /// Names fake tools temporarily hidden from the owned PATH.
    pub(super) unavailable: Vec<Program>,
    /// Assigns or removes explicitly allowlisted child inputs.
    pub(super) environment: Vec<EnvironmentChange>,
    /// Adds one literal extra argument to the entrypoint.
    pub(super) unexpected_argument: bool,
    /// Chooses function-import or startup-file isolation fixtures.
    pub(super) injection: InjectionMode,
    /// Chooses whether successful extraction writes the mold payload.
    pub(super) payload: PayloadMode,
    /// Supplies ordered status and inventory outputs for rustup calls.
    pub(super) rustup: RustupFixture,
    /// Overrides successful fake mold version text when set.
    pub(super) mold_version_output: Option<String>,
    /// Overrides successful fake clang version text when set.
    pub(super) clang_version_output: Option<String>,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            failure: None,
            unavailable: Vec::new(),
            environment: Vec::new(),
            unexpected_argument: false,
            injection: InjectionMode::None,
            payload: PayloadMode::Installed,
            rustup: RustupFixture::default(),
            mold_version_output: None,
            clang_version_output: None,
        }
    }
}

/// Entrypoint, host values, and controlled child inputs.
pub(super) struct Request {
    /// Selects install or check behaviour for the child process.
    pub(super) entry: Entry,
    /// Selects the operating system reported by fake uname.
    pub(super) host: Host,
    /// Selects the machine name reported by fake uname.
    pub(super) machine: String,
    /// Groups the controlled failures, inputs, and executable fixtures.
    pub(super) controls: Controls,
}

impl Request {
    /// Creates the successful pinned Linux baseline.
    pub(super) fn new(entry: Entry) -> Self {
        Self {
            entry,
            host: Host::Linux,
            machine: "x86_64".to_owned(),
            controls: Controls::default(),
        }
    }
}

/// One recorded fake invocation, with exact argv and status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Invocation {
    /// Records the exact name of the invoked fake executable.
    pub(super) tool: String,
    /// Preserves the fake executable arguments in their original order.
    pub(super) argv: Vec<String>,
    /// Records the fake executable status for this call.
    pub(super) status: i32,
}

/// Exact installed state under the owned prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PrefixState {
    /// Reports whether the owned installation prefix directory exists.
    pub(super) exists: bool,
    /// Reports whether prefix/bin/mold has an executable mode bit.
    pub(super) mold_executable: bool,
    /// Retains the exact installed mold payload bytes when present.
    pub(super) mold_bytes: Option<Vec<u8>>,
}

/// Process, fake-call, stdin, and owned-filesystem observations.
pub(super) struct Observation {
    /// Retains the entrypoint or probe process exit status.
    pub(super) status: ExitStatus,
    /// Retains the child standard output bytes without decoding.
    pub(super) stdout: Vec<u8>,
    /// Retains the child standard error bytes without decoding.
    pub(super) stderr: Vec<u8>,
    /// Stores parsed fake calls in their observed execution order.
    pub(super) invocations: Vec<Invocation>,
    /// Retains the exact bytes sent to the fake checksum verifier.
    pub(super) sha_stdin: Vec<u8>,
    /// Retains the exact source bytes sent to fake clang.
    pub(super) clang_stdin: Vec<u8>,
    /// Lists remaining children of the controlled temporary directory.
    pub(super) tmp_children: Vec<String>,
    /// Captures the installed-prefix directory and payload state.
    pub(super) prefix: PrefixState,
    /// Retains any marker written by an imported shadow function.
    pub(super) shadow_marker: Vec<u8>,
}

/// Fixed synthetic Bash function names used by isolation probes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InjectedFunction {
    /// Names the synthetic curl function used to test import isolation.
    Curl,
    /// Names the synthetic uname function used to test import isolation.
    Uname,
    /// Names the synthetic printf function used as a builtin control.
    Printf,
    /// Names the synthetic command function used to test builtin lookup.
    Command,
    /// Names the synthetic mold function used by direct payload probes.
    Mold,
}

impl InjectedFunction {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Curl => "curl",
            Self::Uname => "uname",
            Self::Printf => "printf",
            Self::Command => "command",
            Self::Mold => "mold",
        }
    }
}

/// Entry-level function-import or startup-file fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InjectionMode {
    /// Starts the child without synthetic functions or startup files.
    None,
    /// Supplies the four fixed function-import injections.
    EntryFunctions,
    /// Points child startup variables at the owned exit-marker file.
    StartupFiles,
}

/// Whether successful extraction writes the executable payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PayloadMode {
    /// Makes successful fake extraction copy the executable mold payload.
    Installed,
    /// Makes successful fake extraction leave prefix/bin/mold absent.
    Missing,
}

/// One explicit test-controlled assignment or removal from a child environment.
pub(super) struct EnvironmentChange {
    /// Names one allowlisted production input changed only in the child.
    pub(super) name: String,
    /// Some assigns the child value; None removes that child input.
    pub(super) value: Option<String>,
}

/// Exact output and status fixtures for declared and stable call ordinals.
pub(super) struct RustupFixture {
    /// Statuses for declared-channel then stable install call ordinals.
    pub(super) install_status: [i32; 2],
    /// Statuses for declared-channel then stable list call ordinals.
    pub(super) list_status: [i32; 2],
    /// Toolchain inventory bytes returned for the two list ordinals.
    pub(super) list_output: [String; 2],
    /// Statuses for declared-channel then stable component queries.
    pub(super) component_status: [i32; 2],
    /// Component inventory bytes returned for the two queries.
    pub(super) component_output: [String; 2],
}

impl Default for RustupFixture {
    fn default() -> Self {
        Self {
            install_status: [0, 0],
            list_status: [0, 0],
            list_output: ["1.92.0\n".to_owned(), "1.92.0\n".to_owned()],
            component_status: [0, 0],
            component_output: [
                "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n".to_owned(),
                "rustfmt (installed)\n".to_owned(),
            ],
        }
    }
}

/// Raw status and output returned by one child process.
pub(super) struct RawOutput {
    /// Preserves the process exit status before fake-log decoding.
    pub(super) status: ExitStatus,
    /// Preserves raw standard output bytes from the child process.
    pub(super) stdout: Vec<u8>,
    /// Preserves raw standard error bytes from the child process.
    pub(super) stderr: Vec<u8>,
}

/// Exact fake log and stdin bytes captured from one child process.
pub(super) struct Recordings {
    /// Contains the ordered raw CALL and STATUS records from fake tools.
    pub(super) call_log: Vec<u8>,
    /// Contains the exact checksum input bytes captured from the fake.
    pub(super) sha_stdin: Vec<u8>,
    /// Contains the exact C source bytes captured from fake clang.
    pub(super) clang_stdin: Vec<u8>,
}

/// Normalized path-bearing clones; the original observation remains untouched.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ComparisonEvidence {
    /// Clones calls with only the validated owned download root normalized.
    pub(super) invocations: Vec<Invocation>,
    /// Clones checksum input with the same correlated root normalized.
    pub(super) sha_stdin: Vec<u8>,
}

/// Child-only settings shared by one entrypoint or isolation probe.
pub(super) struct ChildFixture<'a> {
    /// Supplies the operating system reported by the fake uname command.
    pub(super) host: Host,
    /// Supplies the machine reported when the fake uname query is Linux.
    pub(super) machine: &'a str,
    /// Supplies stage failures and explicit child-only input fixtures.
    pub(super) controls: &'a Controls,
    /// Supplies the synthetic Bash functions selected for this launch.
    pub(super) functions: &'a [InjectedFunction],
    /// Enables owned `BASH_ENV` and `ENV` files for startup-file isolation cases.
    pub(super) startup_files: bool,
}
