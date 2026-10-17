//! Classifies exact repository tools and binds UV routes to reviewed Python bodies.

use super::{Route, Workspace};

use python_routes::{classify_uv, is_historical_whitaker, uses_reviewed_python};

/// Stages reviewed bodies for LF/CRLF identity tests.
#[cfg(test)]
pub(super) use python_routes::{LineEnding, stage_python_bodies};
/// Holds the reviewed Python source identities used by workflow routes.
pub(super) use python_routes::{REVIEWED_PYTHON_BODIES, verify_reviewed_python_sources};

/// Resolves UV commands and ties source-sensitive routes to reviewed Python files.
#[path = "ci_reachability_make_python_routes.rs"]
mod python_routes;

/// Probe names are usable only inside Make after its source defaults pass inspection.
pub(super) const PROBE_CARGO: &str = "__gauss_contract_cargo_2b87";
/// Probe name for UV, never an accepted workflow executable.
pub(super) const PROBE_UV: &str = "__gauss_contract_uv_2b87";
/// Probe name for the Whitaker executable.
pub(super) const PROBE_WHITAKER: &str = "__gauss_contract_whitaker_2b87";
/// Probe name for Markdown lint.
pub(super) const PROBE_MDLINT: &str = "__gauss_contract_mdlint_2b87";
/// Probe name for Markdown formatting.
pub(super) const PROBE_MDTABLEFIX: &str = "__gauss_contract_mdtablefix_2b87";
/// Probe name for Mermaid validation.
pub(super) const PROBE_NIXIE: &str = "__gauss_contract_nixie_2b87";

/// Classifies one decoded executable and every argument without consulting source files.
pub(super) fn classify_words(words: &[String], controlled: bool) -> Result<Route, String> {
    let (program, arguments) = words
        .split_first()
        .ok_or_else(|| "empty shell command".to_owned())?;
    match canonical_program(program, controlled) {
        "cargo" => classify_cargo(arguments),
        "uv" => classify_uv(arguments),
        "whitaker" => exact_non_suite(
            arguments,
            &[
                "--all",
                "--",
                "--workspace",
                "--all-targets",
                "--all-features",
            ],
            "Whitaker",
        ),
        "mdtablefix" => exact_non_suite(
            arguments,
            &[
                "--check",
                "--git",
                "--include-untracked",
                "--wrap",
                "--renumber",
                "--breaks",
                "--ellipsis",
                "--fences",
            ],
            "mdtablefix",
        ),
        "markdownlint-cli2" => classify_markdownlint(arguments),
        "nixie" => exact_non_suite(arguments, &["--no-sandbox"], "nixie"),
        "apt-get" => classify_apt(arguments),
        "sudo" => classify_sudo(arguments),
        other => Err(format!("unresolved executable route: {other}")),
    }
}

/// Resolves private probes only after the caller verified the source Make defaults.
fn canonical_program(program: &str, controlled: bool) -> &str {
    if !controlled {
        return program;
    }
    match program {
        PROBE_CARGO => "cargo",
        PROBE_UV => "uv",
        PROBE_WHITAKER => "whitaker",
        PROBE_MDLINT => "markdownlint-cli2",
        PROBE_MDTABLEFIX => "mdtablefix",
        PROBE_NIXIE => "nixie",
        other => other,
    }
}

/// Adds source identity proof to exact UV routes and refuses the absent Whitaker script.
pub(super) fn classify_workspace_words(
    words: &[String],
    controlled: bool,
    workspace: &Workspace<'_>,
) -> Result<Route, String> {
    let route = classify_words(words, controlled)?;
    let (program, arguments) = words
        .split_first()
        .ok_or_else(|| "empty shell command".to_owned())?;
    if canonical_program(program, controlled) == "uv" {
        if is_historical_whitaker(arguments) {
            return Err("Whitaker contract script is absent or unreviewed".to_owned());
        }
        if uses_reviewed_python(arguments) {
            verify_reviewed_python_sources(workspace)?;
        }
    }
    Ok(route)
}

/// Separates suite invocations from reviewed Cargo build, lint and documentation commands.
fn classify_cargo(arguments: &[String]) -> Result<Route, String> {
    match arguments.first().map(String::as_str) {
        Some("nextest") if arguments.get(1).is_some_and(|word| word == "--version") => {
            if arguments.len() == 2 {
                Ok(Route::NonSuite)
            } else {
                Err("unresolved Nextest probe arguments".to_owned())
            }
        }
        Some("nextest") if arguments.get(1).is_some_and(|word| word == "run") => {
            executing_suite(arguments.get(2..).map_or(&[], |rest| rest))
        }
        Some("test") => executing_suite(arguments.get(1..).map_or(&[], |rest| rest)),
        Some("build" | "check" | "clean" | "clippy" | "doc" | "fmt" | "metadata" | "package") => {
            Ok(Route::NonSuite)
        }
        Some(other) => Err(format!("unresolved Cargo subcommand: {other}")),
        None => Err("Cargo executable has no subcommand".to_owned()),
    }
}

/// Rejects Cargo switches that print or compile tests without executing them.
fn executing_suite(arguments: &[String]) -> Result<Route, String> {
    arguments
        .iter()
        .find(|word| {
            matches!(
                word.as_str(),
                "-h" | "--help" | "--no-run" | "--list" | "--dry-run" | "--version"
            )
        })
        .map_or(Ok(Route::Suite), |control| {
            Err(format!(
                "Cargo suite invocation does not execute tests: {control}"
            ))
        })
}

/// Recognizes current Markdown lint commands without admitting extra input globs.
fn classify_markdownlint(arguments: &[String]) -> Result<Route, String> {
    if exact_words(arguments, &["**/*.md"]) || exact_words(arguments, &["--fix", "**/*.md"]) {
        Ok(Route::NonSuite)
    } else {
        Err("unresolved markdownlint-cli2 arguments".to_owned())
    }
}

/// Classifies the only approved apt operations used by the repository workflow.
fn classify_apt(arguments: &[String]) -> Result<Route, String> {
    let install = [
        "install",
        "-y",
        "clang",
        "libxcb1-dev",
        "libxkbcommon-dev",
        "libxkbcommon-x11-dev",
    ];
    if exact_words(arguments, &["update"]) || exact_words(arguments, &install) {
        Ok(Route::NonSuite)
    } else {
        Err("unresolved apt-get invocation".to_owned())
    }
}

/// Restricts sudo to the exact apt operations.
fn classify_sudo(arguments: &[String]) -> Result<Route, String> {
    if arguments
        .first()
        .is_some_and(|program| program == "apt-get")
    {
        classify_apt(arguments.get(1..).map_or(&[], |rest| rest))
    } else {
        Err("unresolved sudo command".to_owned())
    }
}

/// Requires exact argument equality for a reviewed non-suite tool.
fn exact_non_suite(arguments: &[String], expected: &[&str], tool: &str) -> Result<Route, String> {
    if exact_words(arguments, expected) {
        Ok(Route::NonSuite)
    } else {
        Err(format!("unresolved {tool} arguments"))
    }
}

/// Compares all ordered arguments without slicing past the input length.
fn exact_words(actual: &[String], expected: &[&str]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(word, expected_word)| word == expected_word)
}

/// Verifies Python source identities and the current historical script gap.
#[cfg(test)]
#[path = "ci_reachability_make_tools_tests.rs"]
mod tests;
