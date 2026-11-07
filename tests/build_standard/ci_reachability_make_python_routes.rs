//! Checks exact UV routes.

use super::{Route, exact_words};

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
        || is_interrogate(arguments)
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
///
/// Both routes share one pinned dependency set; only the reviewed script list
/// differs, so a new or unpinned dependency changes both routes at once.
fn is_python_test(arguments: &[String]) -> bool {
    let head = [
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
    ];
    let tail = ["-c", "/dev/null", "--rootdir=.", "-p", "no:cacheprovider"];
    let reviewed_routes = [
        ["scripts/tests/test_spelling_exceptions.py"].as_slice(),
        [
            "scripts/tests/test_integration_test_inventory.py",
            "scripts/tests/test_integration_test_inventory_cli.py",
        ]
        .as_slice(),
    ];
    reviewed_routes.into_iter().any(|scripts| {
        exact_words(
            arguments,
            &[head.as_slice(), scripts, tail.as_slice()].concat(),
        )
    })
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

/// Matches the docstring-coverage gate: an `interrogate==<version>` run over
/// plain target paths with the fixed report and threshold flags, and nothing else.
/// The version itself is not compared; the Makefile owns the pin.
pub(super) fn is_interrogate(arguments: &[String]) -> bool {
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let [
        "tool",
        "run",
        "--python",
        "3.14",
        "--from",
        package,
        "interrogate",
        "-vv",
        "--fail-under",
        "100",
        targets @ ..,
    ] = words.as_slice()
    else {
        return false;
    };
    package.starts_with("interrogate==")
        && !targets.is_empty()
        && targets.iter().all(|target| !target.starts_with('-'))
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
