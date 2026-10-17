//! Capability-owned scratch setup, candidate copying, and filesystem observations.
//!
//! Every path mutation remains relative to the opened target child directory;
//! ordinary returns report cleanup errors and Drop only makes a best effort.

use std::{
    io::ErrorKind,
    sync::atomic::{AtomicUsize, Ordering},
};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{
    ambient_authority,
    fs::{Permissions, PermissionsExt},
    fs_utf8::Dir,
};

use super::{
    ALLOWED_UTILITIES, CANDIDATE_FILES, FAKE_COMMON, FAKE_COMMON_FILE, FAKE_FILES, FAKE_MOLD,
    FAKE_MOLD_PAYLOAD_FILE, Observation, PrefixState, ProbeRequest, Program, Request, STARTUP_FILE,
    model::{RawOutput, Recordings},
    process,
    protocol::{self, checked_relative},
};

static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);

/// One isolated candidate tree and all fixtures owned by one contract case.
pub(super) struct Scratch {
    /// Capability for the owned target parent that contains this scratch child.
    pub(super) parent: Dir,
    /// Child name relative to parent, used for precise owned cleanup.
    pub(super) name: Utf8PathBuf,
    /// Open scratch capability, absent after normal cleanup closes it.
    pub(super) dir: Option<Dir>,
    /// Absolute path passed to child processes; edits still use the capability.
    pub(super) path: Utf8PathBuf,
    /// Records completed cleanup so Drop does not remove the child twice.
    cleaned: bool,
}

impl Scratch {
    /// Copies candidate inputs and creates the fake process environment.
    pub(super) fn from_candidate() -> Result<Self, String> {
        let project = Utf8Path::new(env!("CARGO_MANIFEST_DIR"));
        let repository = Dir::open_ambient_dir(project, ambient_authority())
            .map_err(|error| format!("opening repository capability: {error}"))?;
        repository
            .create_dir_all("target/gauss-build-tools-contracts")
            .map_err(|error| format!("creating owned target parent: {error}"))?;
        let parent = repository
            .open_dir("target/gauss-build-tools-contracts")
            .map_err(|error| format!("opening owned target parent: {error}"))?;
        let index = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let name = Utf8PathBuf::from(format!("build-tools-{}-{index}", std::process::id()));
        parent
            .create_dir(name.as_path())
            .map_err(|error| format!("creating owned scratch child: {error}"))?;
        let dir = parent.open_dir(name.as_path()).map_err(|error| {
            let cleanup = parent.remove_dir_all(name.as_path());
            match cleanup {
                Ok(()) => format!("opening owned scratch child: {error}"),
                Err(cleanup_error) => {
                    format!("opening owned scratch child: {error}; cleanup failed: {cleanup_error}")
                }
            }
        })?;
        let path = project
            .join("target/gauss-build-tools-contracts")
            .join(&name);
        let mut scratch = Self {
            parent,
            name,
            dir: Some(dir),
            path,
            cleaned: false,
        };
        scratch.initialize(&repository)?;
        Ok(scratch)
    }

    fn initialize(&mut self, repository: &Dir) -> Result<(), String> {
        let directory = self.directory()?;
        for path in [
            "candidate/scripts",
            "candidate/tools/mold",
            "candidate/tools/rust-stable",
            "bin",
            "fake",
            "state",
            "tmp",
        ] {
            directory
                .create_dir_all(path)
                .map_err(|error| format!("creating controlled directory {path}: {error}"))?;
        }
        self.copy_candidate(repository)?;
        self.install_fakes()?;
        for utility in ALLOWED_UTILITIES {
            let target = Utf8Path::new("/usr/bin").join(utility);
            let link = Utf8PathBuf::from(format!("bin/{utility}"));
            self.directory()?
                .symlink_contents(target, link.as_path())
                .map_err(|error| format!("linking reviewed utility {utility}: {error}"))?;
        }
        self.write_relative("calls.log", b"")?;
        self.write_relative("sha.stdin", b"")?;
        self.write_relative("clang.stdin", b"")?;
        self.write_relative("tmp/unrelated.marker", b"preserve")?;
        self.write_relative(
            STARTUP_FILE,
            b"builtin printf '%s\\n' startup >> \"$GAUSS_SHADOW_LOG\"\nexit 96\n",
        )?;
        Ok(())
    }

    fn copy_candidate(&self, repository: &Dir) -> Result<(), String> {
        for source in CANDIDATE_FILES {
            let destination = format!("candidate/{source}");
            let bytes = repository
                .read(source)
                .map_err(|error| format!("reading candidate {source}: {error}"))?;
            self.directory()?
                .write(&destination, &bytes)
                .map_err(|error| format!("copying candidate {source}: {error}"))?;
        }
        Ok(())
    }

    fn install_fakes(&self) -> Result<(), String> {
        self.write_relative(FAKE_COMMON_FILE, FAKE_COMMON.as_bytes())?;
        for (name, source) in FAKE_FILES {
            let path = format!("bin/{name}");
            self.write_relative(&path, source.as_bytes())?;
            self.directory()?
                .set_permissions(&path, Permissions::from_mode(0o700))
                .map_err(|error| format!("making fake {name} executable: {error}"))?;
        }
        self.write_relative(FAKE_MOLD_PAYLOAD_FILE, FAKE_MOLD.as_bytes())?;
        self.directory()?
            .set_permissions(FAKE_MOLD_PAYLOAD_FILE, Permissions::from_mode(0o700))
            .map_err(|error| format!("making mold payload executable: {error}"))?;
        Ok(())
    }

    fn directory(&self) -> Result<&Dir, String> {
        self.dir
            .as_ref()
            .ok_or_else(|| "owned scratch capability is closed".to_owned())
    }

    /// Writes bytes only at a validated relative path beneath this scratch.
    pub(super) fn write_relative(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        let validated_path = checked_relative(path)?;
        self.directory()?
            .write(validated_path.as_path(), bytes)
            .map_err(|error| format!("writing scratch {validated_path}: {error}"))
    }

    /// Reads exact bytes only at a validated relative path beneath this scratch.
    pub(super) fn read_relative(&self, path: &str) -> Result<Vec<u8>, String> {
        let validated_path = checked_relative(path)?;
        self.directory()?
            .read(validated_path.as_path())
            .map_err(|error| format!("reading scratch {validated_path}: {error}"))
    }

    /// Removes one validated relative file, without traversing outside scratch.
    pub(super) fn remove_relative(&self, path: &str) -> Result<(), String> {
        let validated_path = checked_relative(path)?;
        self.directory()?
            .remove_file(validated_path.as_path())
            .map_err(|error| format!("removing scratch {validated_path}: {error}"))
    }

    /// Runs one production entrypoint and captures process plus owned-state evidence.
    pub(super) fn run(&self, request: &Request) -> Result<Observation, String> {
        self.reset_observations()?;
        let raw = self.with_hidden_tools(&request.controls.unavailable, || {
            process::run_entry(self, request)
        })?;
        self.observe(raw)
    }

    /// Runs one fixed isolation probe against this same owned scratch tree.
    pub(super) fn probe(&self, request: &ProbeRequest) -> Result<Observation, String> {
        self.reset_observations()?;
        self.observe(process::run_probe(self, request)?)
    }

    fn reset_observations(&self) -> Result<(), String> {
        for path in ["calls.log", "sha.stdin", "clang.stdin", "shadow.log"] {
            match self.directory()?.remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(format!("resetting scratch {path}: {error}")),
            }
        }
        for path in [
            "state/rustup-install-count",
            "state/rustup-list-count",
            "state/rustup-component-count",
        ] {
            self.remove_if_present(path)?;
        }
        self.write_relative("calls.log", b"")?;
        self.write_relative("sha.stdin", b"")?;
        self.write_relative("clang.stdin", b"")?;
        Ok(())
    }

    fn observe(&self, raw: RawOutput) -> Result<Observation, String> {
        let recordings = Recordings {
            call_log: self.read_optional("calls.log")?,
            sha_stdin: self.read_optional("sha.stdin")?,
            clang_stdin: self.read_optional("clang.stdin")?,
        };
        let mut observation = protocol::parse_observation(raw, recordings)?;
        observation.tmp_children = self.directory_names("tmp")?;
        observation.prefix = self.prefix_state()?;
        observation.shadow_marker = self.read_optional("shadow.log")?;
        Ok(observation)
    }

    fn prefix_state(&self) -> Result<PrefixState, String> {
        let exists = match self.directory()?.metadata("prefix") {
            Ok(metadata) => metadata.is_dir(),
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => return Err(format!("reading owned prefix state: {error}")),
        };
        let mold_bytes = match self.directory()?.read("prefix/bin/mold") {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(format!("reading installed mold payload: {error}")),
        };
        let mold_executable = if mold_bytes.is_some() {
            self.directory()?
                .metadata("prefix/bin/mold")
                .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
                .map_err(|error| format!("reading installed mold mode: {error}"))?
        } else {
            false
        };
        Ok(PrefixState {
            exists,
            mold_executable,
            mold_bytes,
        })
    }

    fn directory_names(&self, path: &str) -> Result<Vec<String>, String> {
        let entries = self
            .directory()?
            .read_dir(path)
            .map_err(|error| format!("reading owned directory {path}: {error}"))?;
        entries
            .map(|entry| {
                entry
                    .map_err(|error| format!("reading owned directory {path}: {error}"))?
                    .file_name()
                    .map_err(|error| format!("decoding owned entry in {path}: {error}"))
            })
            .collect()
    }

    fn read_optional(&self, path: &str) -> Result<Vec<u8>, String> {
        match self.directory()?.read(path) {
            Ok(bytes) => Ok(bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(format!("reading scratch {path}: {error}")),
        }
    }

    fn remove_if_present(&self, path: &str) -> Result<(), String> {
        match self.directory()?.remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("removing scratch {path}: {error}")),
        }
    }

    fn with_hidden_tools<T>(
        &self,
        unavailable: &[Program],
        operation: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let mut hidden = Vec::new();
        for program in unavailable {
            if hidden.contains(program) {
                continue;
            }
            let source = format!("bin/{}", program.name());
            let target = format!("state/hidden-{}", program.name());
            let directory = self.directory()?;
            if let Err(error) = directory.rename(&source, directory, &target) {
                let restore = self.restore_tools(&hidden);
                return match restore {
                    Ok(()) => Err(format!("hiding fake {}: {error}", program.name())),
                    Err(cleanup) => Err(format!(
                        "hiding fake {}: {error}; restoring fakes: {cleanup}",
                        program.name()
                    )),
                };
            }
            hidden.push(*program);
        }
        let outcome = operation();
        let restore = self.restore_tools(&hidden);
        match (outcome, restore) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
            (Err(primary), Err(cleanup)) => Err(format!("{primary}; restoring fakes: {cleanup}")),
        }
    }

    fn restore_tools(&self, hidden: &[Program]) -> Result<(), String> {
        for program in hidden.iter().rev() {
            let source = format!("state/hidden-{}", program.name());
            let target = format!("bin/{}", program.name());
            let directory = self.directory()?;
            directory
                .rename(&source, directory, &target)
                .map_err(|error| format!("restoring fake {}: {error}", program.name()))?;
        }
        Ok(())
    }

    fn cleanup(mut self) -> Result<(), String> {
        self.dir.take();
        self.parent
            .remove_dir_all(self.name.as_path())
            .map_err(|error| format!("removing owned scratch child: {error}"))?;
        self.cleaned = true;
        Ok(())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.cleaned {
            self.dir.take();
            if let Err(_error) = self.parent.remove_dir_all(self.name.as_path()) {}
        }
    }
}

/// Runs a case with fallible normal cleanup and best-effort cleanup on early exit.
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
            Err(format!("{primary}; cleanup failed: {cleanup_error}"))
        }
    }
}
