//! Discovers Linux suite routes and binds their current same-job `mold` provisioning.
//!
//! This consumer contract inspects the complete local workflow inventory and evaluates Make
//! prerequisites. Its deliberately bounded grammar reports an unmeasured route as an error.
//! The complete build-tool preflight belongs to the later provisioning layer.

use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::fs_utf8::Dir;

#[path = "ci_reachability_lines.rs"]
mod lines;
#[path = "ci_reachability_make_output.rs"]
mod make_output;
#[path = "ci_reachability_make.rs"]
mod make_routes;
#[path = "ci_reachability_make_source.rs"]
mod make_source;
#[path = "ci_reachability_make_syntax.rs"]
mod make_syntax;
#[path = "ci_reachability_make_tools.rs"]
mod make_tools;
#[path = "ci_reachability_matrix.rs"]
mod matrix;
#[path = "ci_reachability_parser.rs"]
mod parser;
#[path = "ci_reachability_runner.rs"]
mod runner;
#[path = "ci_reachability_workflows.rs"]
mod workflows;

/// The currently adopted shared-action revision whose consumed sources were inspected.
const ACTION_REVISION: &str = "e693d81d176d2305e2b736a0be0f9038085dec7f";

/// A strict YAML subset retaining every mapping entry before semantic inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    /// A missing value, including an empty event configuration.
    Empty,
    /// A literal or decoded quoted scalar; expressions remain text until explicitly resolved.
    Scalar(String),
    /// An ordered sequence.
    Sequence(Vec<Self>),
    /// A mapping whose parser has already refused duplicate decoded keys.
    Mapping(BTreeMap<String, Self>),
}

impl Value {
    /// Reads a literal scalar, for example `Scalar("main").text("ref")` yields `"main"`.
    fn text(&self, context: &str) -> Result<&str, String> {
        match self {
            Self::Scalar(text) => Ok(text),
            _ => Err(format!("{context}: expected a scalar")),
        }
    }

    /// Reads a mapping, refusing a sequence where an object was required.
    fn mapping(&self, context: &str) -> Result<&BTreeMap<String, Self>, String> {
        match self {
            Self::Mapping(entries) => Ok(entries),
            _ => Err(format!("{context}: expected a mapping")),
        }
    }

    /// Reads an ordered sequence without treating a scalar as a singleton list.
    fn sequence(&self, context: &str) -> Result<&[Self], String> {
        match self {
            Self::Sequence(items) => Ok(items),
            _ => Err(format!("{context}: expected a sequence")),
        }
    }

    /// Finds an optional mapping field, for example the `runs-on` field of a job.
    fn field(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Mapping(entries) => entries.get(key),
            _ => None,
        }
    }

    /// Requires a mapping field, naming the missing path rather than silently omitting it.
    fn required(&self, key: &str, context: &str) -> Result<&Self, String> {
        self.mapping(context)?
            .get(key)
            .ok_or_else(|| format!("{context}: missing {key}"))
    }

    /// Decodes an optional scalar field while still refusing a malformed present field.
    fn optional_text(&self, key: &str, context: &str) -> Result<Option<&str>, String> {
        self.field(key).map(|value| value.text(context)).transpose()
    }
}

/// The operating-system evidence supplied by a statically resolved runner row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Host {
    /// Linux requires the current pinned `mold` provisioner.
    Linux,
    /// macOS is a legitimate non-Linux exception.
    Darwin,
    /// Windows is a legitimate non-Linux exception.
    Windows,
}

impl Host {
    /// Supplies Make's supported host discriminator for a controlled dry run.
    const fn make_value(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Darwin => "Darwin",
            Self::Windows => "Windows_NT",
        }
    }
}

/// Whether a completely inspected shell/action/Make route reaches a Rust test suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    /// Every executable command was resolved and none runs a suite.
    NonSuite,
    /// The route runs a suite, including doctests.
    Suite,
}

impl Route {
    /// Combines resolved commands without losing a suite reached through a prerequisite.
    const fn combine(self, other: Self) -> Self {
        if matches!(self, Self::Suite) || matches!(other, Self::Suite) {
            Self::Suite
        } else {
            Self::NonSuite
        }
    }
}

/// A capability-scoped source root and the same root's process working directory.
struct Workspace<'a> {
    /// Capability used for every source and fixture read.
    directory: &'a Dir,
    /// Explicit process boundary; subprocesses cannot derive an ambient working directory.
    path: &'a Utf8Path,
}

/// A decoded shell invocation retaining environment and redirection evidence.
#[derive(Clone, Debug)]
struct Invocation {
    /// Executable followed by every ordered argument.
    words: Vec<String>,
    /// Leading environment assignments, before any classification removes them.
    assignments: Vec<String>,
    /// Redirections whose disposition the command classifier must prove.
    redirects: Vec<String>,
}

impl Invocation {
    /// Checks only the current consumer's explicit environment boundary.
    /// For example, two `PYTHONPATH` assignments produce a duplicate-name error.
    fn validate_environment(&self) -> Result<(), String> {
        let mut names = std::collections::BTreeSet::new();
        for assignment in &self.assignments {
            let (name, value) = assignment
                .split_once('=')
                .ok_or_else(|| "malformed shell environment assignment".to_owned())?;
            if !names.insert(name) {
                return Err(format!("duplicate shell environment assignment: {name}"));
            }
            if !Self::allows_assignment(name, value) {
                return Err(format!("unreviewed shell environment assignment: {name}"));
            }
        }
        Ok(())
    }

    /// Keeps compiler-token whitespace flexible while every other value remains exact.
    /// For example, `PYTHONPATH=scripts` passes, whereas `PYTHONPATH=other` fails.
    fn allows_assignment(name: &str, value: &str) -> bool {
        match name {
            "RUSTFLAGS" => Self::allows_rustflags(value),
            "RUSTDOCFLAGS" => value
                .split_whitespace()
                .eq(["--cfg", "docsrs", "-D", "warnings"]),
            "PYTHONPATH" => value == "scripts",
            "HYPOTHESIS_STORAGE_DIRECTORY" => value == "/tmp/gauss-hypothesis",
            "UV_CACHE_DIR" => value == ".uv-cache",
            "UV_TOOL_DIR" => value == ".uv-tools",
            _ => false,
        }
    }

    /// Retains the inherited expansion and complete ordered development suffix.
    /// For example, duplicate `-D warnings` suffixes fail instead of losing a token.
    /// Recognizing a suffix does not qualify its backend or linker for a host.
    fn allows_rustflags(value: &str) -> bool {
        let inherited = ["$", "{RUSTFLAGS:+$RUSTFLAGS }"].concat();
        let release = ["$", "{RUSTFLAGS-}"].concat();
        if value == release {
            return true;
        }
        let Some(suffix) = value.strip_prefix(&inherited) else {
            return false;
        };
        let tokens = suffix.split_whitespace().collect::<Vec<_>>();
        matches!(
            tokens.as_slice(),
            [] | ["-D", "warnings"]
                | ["-D", "warnings", "-Clink-arg=-fuse-ld=mold"]
                | ["-Clink-arg=-fuse-ld=mold"]
        )
    }
}

impl Workspace<'_> {
    /// Verifies one reviewed source body through this capability, never its filename alone.
    /// For example, changing a known module's body produces its identity-change error.
    fn verify_source(&self, path: &str, expected: &str) -> Result<(), String> {
        let source = self
            .directory
            .read_to_string(Utf8Path::new(path))
            .map_err(|error| format!("reading reviewed Python source {path}: {error}"))?;
        let normalized = Self::normalize_source(&source);
        let observed = self
            .git_blob_id(&normalized)
            .map_err(|error| format!("hashing reviewed Python source {path}: {error}"))?;
        if observed == expected {
            Ok(())
        } else {
            Err(format!("reviewed Python source identity changed: {path}"))
        }
    }

    /// Mirrors Python text mode: CRLF and lone carriage returns become LF.
    /// For example, `"a\r\nb\r"` becomes `"a\nb\n"`.
    fn normalize_source(source: &str) -> String {
        source.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Hashes stdin without writing Git objects, filters, paths or network access.
    /// For example, a failed Git exit returns its status and stderr as an error.
    fn git_blob_id(&self, source: &str) -> Result<String, String> {
        let mut child = Command::new("/usr/bin/git")
            .args(["hash-object", "--stdin"])
            .current_dir(self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("starting git hash-object --stdin: {error}"))?;
        let write_result = child
            .stdin
            .take()
            .ok_or_else(|| "git hash-object stdin pipe is missing".to_owned())
            .and_then(|mut input| {
                input
                    .write_all(source.as_bytes())
                    .map_err(|error| format!("writing Git hash input: {error}"))
            });
        let waited = child
            .wait_with_output()
            .map_err(|error| format!("waiting for git hash-object: {error}"));
        let output = match (write_result, waited) {
            (Err(write_error), Err(wait_error)) => {
                return Err(format!("{write_error}; {wait_error}"));
            }
            (Err(write_error), Ok(output)) => {
                return Err(format!(
                    "{write_error}; git exited {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
            (Ok(()), Err(wait_error)) => return Err(wait_error),
            (Ok(()), Ok(output)) => output,
        };
        if !output.status.success() {
            return Err(format!(
                "git hash-object failed with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let hash_bytes = output
            .stdout
            .strip_suffix(b"\n")
            .ok_or_else(|| "git hash-object output has no terminal newline".to_owned())?;
        let hash_text = std::str::from_utf8(hash_bytes)
            .map_err(|error| format!("git hash-object returned non-UTF-8 output: {error}"))?;
        if hash_text.len() == 40
            && hash_text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(hash_text.to_owned())
        } else {
            Err("git hash-object returned an invalid blob identifier".to_owned())
        }
    }
}

/// A parsed local workflow retaining its inventory identity for reusable-call resolution.
#[derive(Clone, Debug)]
struct Workflow {
    /// Repository-relative `.github/workflows` path.
    path: Utf8PathBuf,
    /// The complete strict document, including keys not directly related to suites.
    document: Value,
}

/// Non-vacuous inspection counts, returned only after every inventory entry has been resolved.
#[derive(Debug, PartialEq, Eq)]
struct Coverage {
    /// Number of event-reachable jobs with a suite route, including non-Linux jobs.
    suite_jobs: usize,
    /// Number of event-reachable Linux runner rows requiring `mold` before their suite route.
    linux_rows: usize,
}

/// Evaluates every local workflow against the current consumer installation contract.
fn inspect(workspace: &Workspace<'_>) -> Result<Coverage, String> {
    let inventory = workflows::read_inventory(workspace)?;
    workflows::inspect_inventory(&inventory, workspace)
}

#[cfg(test)]
#[path = "ci_reachability_make_tests.rs"]
mod make_tests;
#[cfg(test)]
#[path = "ci_reachability_matrix_tests.rs"]
mod matrix_tests;
#[cfg(test)]
#[path = "ci_reachability_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "ci_reachability_workflows_tests.rs"]
mod workflow_tests;
