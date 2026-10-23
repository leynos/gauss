//! Checks reviewed Python source identities and exact UV routes.

use super::super::ReviewedSource;
use super::{Route, Workspace, exact_words};

/// Content identities that authorize the exact current Python inventory and spelling routes.
pub(in super::super) const REVIEWED_PYTHON_BODIES: &[ReviewedSource<'static>] = &[
    ReviewedSource {
        path: "scripts/check_integration_test_inventory.py",
        expected_blob: "d5691f3985e588617e986959738fbbbc7f3dc31f",
    },
    ReviewedSource {
        path: "scripts/tests/test_integration_test_inventory.py",
        expected_blob: "2eef53e6f542fe5731e72721ba6cec1b1dd30be6",
    },
    ReviewedSource {
        path: "scripts/tests/test_integration_test_inventory_cli.py",
        expected_blob: "f9bc11b560144de122b79ce5affc3081b293c0a6",
    },
    ReviewedSource {
        path: "scripts/tests/test_spelling_exceptions.py",
        expected_blob: "14c1f91c4dc7c523c5a95d4ce942a504885e889f",
    },
];

/// Line endings used to prove universal-newline identity equivalence in owned fixtures.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(in super::super) enum LineEnding {
    /// Native text-mode input with LF line endings.
    Lf,
    /// Equivalent source represented with CRLF line endings.
    CrLf,
}

/// Stages every reviewed script body under a fixture capability with the selected line endings.
#[cfg(test)]
pub(in super::super) fn stage_python_bodies(
    workspace: &Workspace<'_>,
    ending: LineEnding,
) -> Result<(), String> {
    let project = super::super::tests::project_directory()?;
    for reviewed in REVIEWED_PYTHON_BODIES {
        let path = reviewed.path();
        let body = project
            .read_to_string(path)
            .map_err(|error| format!("reading reviewed Python source {path}: {error}"))?;
        let normalized = Workspace::normalize_source(&body);
        let staged = match ending {
            LineEnding::Lf => normalized,
            LineEnding::CrLf => normalized.replace('\n', "\r\n"),
        };
        let parent = path
            .parent()
            .ok_or_else(|| format!("reviewed source has no parent directory: {path}"))?;
        workspace
            .directory
            .create_dir_all(parent)
            .map_err(|error| format!("creating reviewed source directory: {error}"))?;
        workspace
            .directory
            .write(path, staged)
            .map_err(|error| format!("staging reviewed Python source {path}: {error}"))?;
    }
    Ok(())
}

/// Keeps the four known body paths tied to reviewed content rather than a filename allowlist.
pub(in super::super) fn verify_reviewed_python_sources(
    workspace: &Workspace<'_>,
) -> Result<(), String> {
    for reviewed in REVIEWED_PYTHON_BODIES {
        workspace.verify_source(reviewed)?;
    }
    Ok(())
}

/// Recognizes only current exact UV routes; unfamiliar versions and scripts remain gaps.
pub(super) fn classify_uv(arguments: &[String]) -> Result<Route, String> {
    if is_reviewed_uv_route(arguments) {
        Ok(Route::NonSuite)
    } else if let Some(reason) = whitaker_route_gap(arguments) {
        Err(reason.to_owned())
    } else {
        Err("unresolved UV command, version, module, or script".to_owned())
    }
}

/// Checks exact reviewed UV routes in order and stops at the first match.
/// For example, a checker match returns `true` without probing later contract routes.
fn is_reviewed_uv_route(arguments: &[String]) -> bool {
    is_checker(arguments)
        || is_python_test(arguments)
        || is_ruff(arguments)
        || is_spelling_builder(arguments)
        || is_coverage_contract(arguments)
        || is_historical_whitaker(arguments)
}

/// Names the specific unsupported historical Whitaker spelling without admitting it.
fn whitaker_route_gap(arguments: &[String]) -> Option<&'static str> {
    if arguments
        .iter()
        .any(|word| word == "scripts/run_whitaker.py")
    {
        return Some("opaque Whitaker Python script is unreviewed");
    }
    if !arguments
        .iter()
        .any(|word| word == "scripts/tests/test_whitaker_contracts.py")
    {
        return None;
    }
    if arguments
        .iter()
        .any(|word| word == "scripts/tests/extra.py")
    {
        return Some("Whitaker command has an unreviewed extra script");
    }
    if !arguments.iter().any(|word| word == "pytest==9.0.2") {
        return Some("Whitaker command does not pin pytest 9.0.2");
    }
    if !arguments
        .iter()
        .any(|word| word.eq_ignore_ascii_case("pyyaml==6.0.3"))
    {
        return Some("Whitaker command does not pin PyYAML 6.0.3");
    }
    None
}

/// Recognizes the direct integration inventory checker.
fn is_checker(arguments: &[String]) -> bool {
    exact_words(
        arguments,
        &["run", "scripts/check_integration_test_inventory.py"],
    )
}

/// Recognizes only the two repository-pinned pytest routes and their complete selectors.
fn is_python_test(arguments: &[String]) -> bool {
    let spelling = [
        "run",
        "--no-project",
        "--python",
        "3.13",
        "--with",
        "pytest==9.0.2",
        "python",
        "-m",
        "pytest",
        "scripts/tests/test_spelling_exceptions.py",
        "-c",
        "/dev/null",
        "--rootdir=.",
        "-p",
        "no:cacheprovider",
    ];
    let inventory = [
        "run",
        "--no-project",
        "--python",
        "3.13",
        "--with",
        "pytest==9.0.2",
        "--with",
        "hypothesis==6.151.9",
        "python",
        "-m",
        "pytest",
        "scripts/tests/test_integration_test_inventory.py",
        "scripts/tests/test_integration_test_inventory_cli.py",
        "-c",
        "/dev/null",
        "--rootdir=.",
        "-p",
        "no:cacheprovider",
    ];
    exact_words(arguments, &spelling) || exact_words(arguments, &inventory)
}

/// Matches only isolated Ruff format and lint commands over the reviewed inventory sources.
fn is_ruff(arguments: &[String]) -> bool {
    let head = ["tool", "run", "ruff@0.15.12"];
    let [checker, inventory_tests, cli_tests] = [
        "scripts/check_integration_test_inventory.py",
        "scripts/tests/test_integration_test_inventory.py",
        "scripts/tests/test_integration_test_inventory_cli.py",
    ];
    ["format", "check"].iter().any(|mode| {
        let mut tail = if *mode == "format" {
            vec![
                "format",
                "--isolated",
                "--target-version",
                "py313",
                "--check",
            ]
        } else {
            vec!["check", "--isolated", "--target-version", "py313"]
        };
        tail.extend([checker, inventory_tests, cli_tests]);
        exact_words(arguments, &[head.as_slice(), tail.as_slice()].concat())
    })
}

/// Matches the pinned spelling builder release and its required gate command.
fn is_spelling_builder(arguments: &[String]) -> bool {
    exact_words(
        arguments,
        &[
            "tool",
            "run",
            "--python",
            "3.14",
            "--from",
            "git+https://github.com/leynos/typos-config-builder.git@v0.1.3",
            "typos-config-builder",
            "gate",
            "--repository",
            ".",
        ],
    )
}

/// Matches the current full-SHA CV-005 contract package and command.
fn is_coverage_contract(arguments: &[String]) -> bool {
    exact_words(
        arguments,
        &[
            "tool",
            "run",
            "--python",
            "3.13",
            "--from",
            "git+https://github.com/leynos/shared-actions@88977798a5c3bae1549afb99642529488c665276#subdirectory=packages/cv005-contracts",
            "cv005-contracts",
            "check",
            "--repository",
            ".",
        ],
    )
}

/// Accepts only the historical lexical case; production classification refuses its missing body.
pub(super) fn is_historical_whitaker(arguments: &[String]) -> bool {
    exact_words(
        arguments,
        &[
            "run",
            "--no-project",
            "--python",
            "3.13",
            "--with",
            "pytest==9.0.2",
            "--with",
            "pyyaml==6.0.3",
            "python",
            "-m",
            "pytest",
            "scripts/tests/test_whitaker_contracts.py",
            "-c",
            "/dev/null",
            "--rootdir=.",
            "-p",
            "no:cacheprovider",
        ],
    )
}

/// Identifies which exact UV commands consume one or more reviewed Python bodies.
pub(super) fn uses_reviewed_python(arguments: &[String]) -> bool {
    is_checker(arguments) || is_python_test(arguments) || is_ruff(arguments)
}
