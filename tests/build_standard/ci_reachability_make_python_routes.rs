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

/// Recognizes only the repository-pinned pytest routes and their complete selectors.
///
/// Every route shares one pinned dependency set; only the reviewed script list
/// differs, so a new or unpinned dependency changes all routes at once.
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
    is_pinned_pytest(arguments, &head, &tail, REVIEWED_PYTEST_SCRIPTS)
}

/// Every reviewed pytest script set, each one an exact route.
///
/// A set is admitted only as written: adding a script to a route, or
/// introducing a route whose scripts are not listed here, remains a gap. The
/// release publish-intent route earns its place because that resolver gates
/// the release job's `if:`, so a silent regression there would upload assets.
const REVIEWED_PYTEST_SCRIPTS: &[&[&str]] = &[
    &["scripts/tests/test_spelling_exceptions.py"],
    &[
        "scripts/tests/test_integration_test_inventory.py",
        "scripts/tests/test_integration_test_inventory_cli.py",
    ],
    &["scripts/tests/test_release_publish_intent.py"],
];

/// Matches a pytest route built from the shared prefix, a reviewed script set,
/// and the shared selector tail.
fn is_pinned_pytest(
    arguments: &[String],
    head: &[&str],
    tail: &[&str],
    reviewed_routes: &[&[&str]],
) -> bool {
    reviewed_routes
        .iter()
        .any(|scripts| exact_words(arguments, &[head, *scripts, tail].concat()))
}

/// Every source set a reviewed Ruff command may cover, each one exact.
///
/// The inventory checker and the publish-intent resolver are formatted and
/// linted in their own targets, so each target contributes its own set. Both
/// modes of both routes are reviewed per set rather than as one union, which
/// keeps a source added to one target from silently widening the other.
const REVIEWED_RUFF_SOURCES: &[&[&str]] = &[
    &[
        "scripts/check_integration_test_inventory.py",
        "scripts/tests/test_integration_test_inventory.py",
        "scripts/tests/test_integration_test_inventory_cli.py",
    ],
    &[
        "scripts/release_publish_intent.py",
        "scripts/tests/test_release_publish_intent.py",
    ],
];

/// Matches only isolated Ruff format and lint commands over the reviewed sources.
fn is_ruff(arguments: &[String]) -> bool {
    REVIEWED_RUFF_SOURCES
        .iter()
        .any(|sources| is_pinned_ruff(arguments, sources))
}

/// Matches one Ruff route: the pinned release, a reviewed mode, and an exact
/// source set.
///
/// `format` carries `--check` because the formatter gate verifies rather than
/// rewrites; `check` has no such flag. Keeping both spellings here means a new
/// source set cannot accidentally acquire a route the Makefile does not run.
fn is_pinned_ruff(arguments: &[String], sources: &[&str]) -> bool {
    let head = ["tool", "run", "ruff@0.15.12"];
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
        tail.extend_from_slice(sources);
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
