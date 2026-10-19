//! Runs selected Make recipes against fake tools and records their exact inputs.
//!
//! Each invocation copies the candidate Makefile below `target`, so real Make
//! evaluates its branches without invoking a compiler, suite, or installer.

use std::{
    process::{Command, ExitStatus},
    sync::atomic::{AtomicUsize, Ordering},
};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};

/// Source module for the immutable shell fixtures used by controlled Make runs.
#[path = "make_input_fixture.rs"]
mod fixture;

use self::fixture::{CARGO_RECORDER, RECURSIVE_FIXTURE, UV_NOOP};

/// Synthetic values used to distinguish inherited and recipe-assigned flags.
pub(super) const INHERITED_RUSTFLAGS: &str = "--cfg=gauss_inherited_rustflags";
/// Synthetic inherited rustdoc flags, distinct from Make's docs flags.
pub(super) const INHERITED_RUSTDOCFLAGS: &str = "--cfg=gauss_inherited_rustdocflags";
/// The documented values assigned by Make's test and lint documentation routes.
pub(super) const RESTATED_RUSTDOCFLAGS: &str = "--cfg docsrs -D warnings";
/// The non-default options supplied only to Nextest run commands.
pub(super) const NEXTEST_OPTIONS: &str = "--cargo-verbose --cargo-verbose";
/// The competing inherited value used to prove a Make command-line assignment wins.
pub(super) const COMPETING_RUST_FLAGS: &str = "--cfg=gauss_environment_value_loses_to_cli";

/// Makes scratch directory names unique across concurrently running test cases.
static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);

/// The caller-controlled origin of Make's `RUST_FLAGS` input.
#[derive(Clone, Copy, Debug)]
pub(super) enum RustFlagsInput {
    /// No caller value is supplied.
    Default,
    /// A synthetic environment value is supplied.
    Environment,
    /// A Make command-line assignment replaces the environment value.
    CommandLine,
}

/// Inputs controlling one Make run and its fake tool behaviour.
pub(super) struct RunRequest<'a> {
    /// Target whose real Make recipe is evaluated.
    pub(super) target: &'a str,
    /// Host value used to select platform flags.
    pub(super) host: super::make::Host,
    /// Caller origin for `RUST_FLAGS`.
    pub(super) rust_flags: RustFlagsInput,
    /// Whether the fake Nextest version check succeeds.
    pub(super) nextest_available: bool,
    /// Synthetic options passed to Nextest runs.
    pub(super) nextest_flags: &'a str,
    /// Name of the controlled failing command, or an empty string.
    pub(super) failure: &'a str,
}

/// One fake-tool invocation recorded by the shell recorder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Invocation {
    /// Exact arguments received by the fake tool.
    pub(super) arguments: Vec<String>,
    /// Whether the Make-only variable leaked into the tool environment.
    pub(super) rust_flags_present: bool,
    /// Effective compiler flags received by the fake tool.
    pub(super) rustflags: String,
    /// Effective rustdoc flags received by the fake tool.
    pub(super) rustdocflags: String,
}

/// Result and recorder output from one Make target.
pub(super) struct Observation {
    /// Exit status returned by real Make.
    pub(super) status: ExitStatus,
    /// Fake Cargo or Whitaker invocations, in execution order.
    pub(super) calls: Vec<Invocation>,
}

/// A copied candidate Makefile and its capability-scoped fake tools.
pub(super) struct Scratch {
    /// Capability for removing the unique child directory.
    parent: Dir,
    /// Child directory name relative to `parent`.
    name: Utf8PathBuf,
    /// Capability rooted at the child directory.
    dir: Option<Dir>,
    /// Absolute path used only as Make's current directory.
    path: Utf8PathBuf,
    /// Whether the test completed its fallible cleanup.
    cleaned: bool,
}

impl Scratch {
    /// Copies the candidate Makefile and creates controlled recorder files.
    pub(super) fn from_candidate() -> Result<Self, String> {
        let project = Utf8Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = Dir::open_ambient_dir(project, ambient_authority())
            .map_err(|error| format!("opening repository capability: {error}"))?;
        let makefile = root
            .read_to_string("Makefile")
            .map_err(|error| format!("reading candidate Makefile: {error}"))?;
        root.create_dir_all("target/gauss-make-input-contracts")
            .map_err(|error| format!("creating owned target directory: {error}"))?;
        let parent = root
            .open_dir("target/gauss-make-input-contracts")
            .map_err(|error| format!("opening owned target directory: {error}"))?;
        let index = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let name = Utf8PathBuf::from(format!("make-{}-{index}", std::process::id()));
        parent
            .create_dir(name.as_path())
            .map_err(|error| format!("creating controlled Make directory: {error}"))?;
        let dir = parent.open_dir(name.as_path()).map_err(|error| {
            match parent.remove_dir_all(name.as_path()) {
                Ok(()) => format!("opening controlled Make directory: {error}"),
                Err(cleanup_error) => format!(
                    "opening controlled Make directory: {error}; cleanup failed: {cleanup_error}"
                ),
            }
        })?;
        let path = project
            .join("target/gauss-make-input-contracts")
            .join(&name);
        let scratch = Self {
            parent,
            name,
            dir: Some(dir),
            path,
            cleaned: false,
        };
        for (fixture_path, fixture_contents) in [
            ("Makefile", makefile.as_bytes()),
            ("cargo-recorder.sh", CARGO_RECORDER.as_bytes()),
            ("uv-noop.sh", UV_NOOP.as_bytes()),
            ("calls.log", b"".as_slice()),
        ] {
            scratch
                .directory()?
                .write(fixture_path, fixture_contents)
                .map_err(|error| format!("writing controlled {fixture_path}: {error}"))?;
        }
        Ok(scratch)
    }

    /// Returns the live capability, failing after cleanup rather than panicking.
    fn directory(&self) -> Result<&Dir, String> {
        self.dir
            .as_ref()
            .ok_or_else(|| "controlled Make directory is closed".to_owned())
    }

    /// Removes the scratch tree and reports cleanup failure on normal test completion.
    fn cleanup(mut self) -> Result<(), String> {
        self.dir.take();
        self.parent
            .remove_dir_all(self.name.as_path())
            .map_err(|error| format!("removing controlled Make directory: {error}"))?;
        self.cleaned = true;
        Ok(())
    }

    /// Replaces only the scratch copy with one source mutation.
    pub(super) fn write_makefile(&self, contents: &str) -> Result<(), String> {
        self.directory()?
            .write("Makefile", contents.as_bytes())
            .map_err(|error| format!("writing scratch Makefile: {error}"))
    }

    /// Reads the scratch Makefile for a unique source-copy mutation.
    pub(super) fn read_makefile(&self) -> Result<String, String> {
        self.directory()?
            .read_to_string("Makefile")
            .map_err(|error| format!("reading scratch Makefile: {error}"))
    }

    /// Adds a recursive fixture proving command-line variables travel in MAKEFLAGS.
    pub(super) fn add_recursive_fixture(&self) -> Result<(), String> {
        let mut makefile = self
            .directory()?
            .read_to_string("Makefile")
            .map_err(|error| format!("reading scratch Makefile: {error}"))?;
        makefile.push_str(RECURSIVE_FIXTURE);
        self.write_makefile(&makefile)
    }

    /// Evaluates a real Make target while replacing every executable tool with a recorder.
    pub(super) fn run(&self, request: &RunRequest<'_>) -> Result<Observation, String> {
        self.directory()?
            .write("calls.log", b"")
            .map_err(|error| format!("resetting recorder log: {error}"))?;
        let mut command = self.command_for(request);
        let output = command
            .output()
            .map_err(|error| format!("running controlled make {}: {error}", request.target))?;
        let log = self
            .directory()?
            .read_to_string("calls.log")
            .map_err(|error| format!("reading recorder log: {error}"))?;
        Ok(Observation {
            status: output.status,
            calls: parse_calls(&log)?,
        })
    }

    /// Builds the controlled Make command without executing it.
    fn command_for(&self, request: &RunRequest<'_>) -> Command {
        let tool = |name: &str| {
            format!(
                "{name}=sh {}",
                shell_word(&self.path.join("cargo-recorder.sh"))
            )
        };
        let mut command = Command::new("make");
        command.args(["--no-print-directory", "-B", "-j1", "-f", "Makefile"]);
        command.args([
            format!("BUILD_HOST_OS={}", request.host.make_value()),
            tool("CARGO"),
            format!("UV=sh {}", shell_word(&self.path.join("uv-noop.sh"))),
            tool("WHITAKER"),
            "TEST_FLAGS=--workspace --all-targets --all-features".to_owned(),
            "CARGO_FLAGS=--workspace --all-targets --all-features".to_owned(),
            "BUILD_JOBS=-j1".to_owned(),
            "RUSTDOC_FLAGS=--cfg docsrs -D warnings".to_owned(),
            format!("NEXTEST_FLAGS={}", request.nextest_flags),
        ]);
        if let RustFlagsInput::CommandLine = request.rust_flags {
            command.arg("RUST_FLAGS=-D warnings --cfg=gauss_rust_flags_marker");
        }
        command
            .arg(request.target)
            .current_dir(self.path.as_path())
            .env(
                "GAUSS_MAKE_CONTRACT_LOG",
                self.path.join("calls.log").as_str(),
            )
            .env(
                "GAUSS_NEXTTEST_AVAILABLE",
                if request.nextest_available {
                    "yes"
                } else {
                    "no"
                },
            )
            .env("GAUSS_FAIL_COMMAND", request.failure)
            .env("RUSTFLAGS", INHERITED_RUSTFLAGS)
            .env("RUSTDOCFLAGS", INHERITED_RUSTDOCFLAGS);
        // A surrounding `make test` owns these inputs. Removing only its Make
        // forwarding variables isolates this first child; recursive Make then
        // forwards the controlled command-line assignment generated above.
        for name in [
            "MAKEFLAGS",
            "MAKEOVERRIDES",
            "MAKEFILES",
            "GNUMAKEFLAGS",
            "MFLAGS",
            "RUST_FLAGS",
            "CARGO",
            "UV",
            "WHITAKER",
            "TEST_FLAGS",
            "CARGO_FLAGS",
            "CLIPPY_FLAGS",
            "BUILD_JOBS",
            "RUSTDOC_FLAGS",
            "NEXTEST_FLAGS",
            "MDTABLEFIX",
            "MDLINT",
            "NIXIE",
            "TYPOS_CONFIG_BUILDER",
        ] {
            command.env_remove(name);
        }
        if !matches!(request.rust_flags, RustFlagsInput::Default) {
            command.env("RUST_FLAGS", COMPETING_RUST_FLAGS);
        }
        command
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.cleaned {
            // Normal returns use fallible cleanup; this covers early exits and unwinding.
            self.dir.take();
            if let Err(_error) = self.parent.remove_dir_all(self.name.as_path()) {}
        }
    }
}

/// Runs a contract with fallible normal cleanup and a best-effort drop on early errors or unwinding.
pub(super) fn with_scratch<T>(
    operation: impl FnOnce(&Scratch) -> Result<T, String>,
) -> Result<T, String> {
    let scratch = Scratch::from_candidate()?;
    let outcome = operation(&scratch);
    let cleanup_result = scratch.cleanup();
    match (outcome, cleanup_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(primary), Err(cleanup_error)) => {
            Err(format!("{primary}; cleanup failed: {cleanup_error}",))
        }
    }
}

/// Quotes a path as one shell word for the fake executable variable.
fn shell_word(path: &Utf8Path) -> String {
    format!("'{}'", path.as_str().replace('\'', "'\\''"))
}

/// Parses the tab-delimited recorder output, refusing absent or unknown fields.
fn parse_calls(log: &str) -> Result<Vec<Invocation>, String> {
    let mut calls = Vec::new();
    for (index, line) in log.lines().enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.first() != Some(&"CALL") {
            return Err(format!("unexpected recorder row {}: {line}", index + 1));
        }
        let offset = fields
            .iter()
            .position(|field| field.starts_with("ENV_RUST_FLAGS="))
            .ok_or_else(|| format!("recorder row {} omits its environment", index + 1))?;
        if fields.len() != offset + 3 {
            return Err(format!(
                "recorder row {} has extra or missing fields",
                index + 1
            ));
        }
        let flag_state = fields
            .get(offset)
            .copied()
            .ok_or_else(|| "missing RUST_FLAGS state".to_owned())?;
        let rust_flags_present = match flag_state {
            "ENV_RUST_FLAGS=present" => true,
            "ENV_RUST_FLAGS=" => false,
            value => return Err(format!("unknown RUST_FLAGS state: {value}")),
        };
        let arguments = fields
            .get(1..offset)
            .ok_or_else(|| "missing fake-tool arguments".to_owned())?
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect();
        calls.push(Invocation {
            arguments,
            rust_flags_present,
            rustflags: field_value(fields.get(offset + 1), "ENV_RUSTFLAGS=")?,
            rustdocflags: field_value(fields.get(offset + 2), "ENV_RUSTDOCFLAGS=")?,
        });
    }
    if calls.is_empty() {
        return Err("Make recorded no fake-tool invocation".to_owned());
    }
    Ok(calls)
}

/// Reads one recorder environment field after checking its expected prefix.
fn field_value(recorded_field: Option<&&str>, prefix: &str) -> Result<String, String> {
    let recorded_value =
        recorded_field.ok_or_else(|| format!("missing recorder field {prefix}"))?;
    recorded_value
        .strip_prefix(prefix)
        .map(str::to_owned)
        .ok_or_else(|| format!("recorder field does not start with {prefix}"))
}

/// Splits a controlled argument string into the argv words Make should produce.
pub(super) fn arguments(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

/// Checks every command, argv and compiler environment for a supported Make route.
pub(super) fn check_target(scratch: &Scratch, request: &RunRequest<'_>) -> Result<(), String> {
    let observation = scratch.run(request)?;
    if !observation.status.success() {
        return Err(format!(
            "make {} failed on the controlled route",
            request.target
        ));
    }
    let expected = super::rust_flags::cases::expected_calls(request)?;
    super::rust_flags::require_calls(
        &expected,
        &observation.calls,
        &format!("make {}", request.target),
    )
}
