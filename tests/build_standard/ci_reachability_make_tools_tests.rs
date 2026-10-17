//! Specifies reviewed Python identities and the historical Whitaker source gap.

use rstest::rstest;

use super::super::tests as shared_tests;
use super::super::{GitBlobId, Route, make_syntax};
use super::{
    LineEnding, REVIEWED_PYTHON_BODIES, classify_words, classify_workspace_words,
    stage_python_bodies, verify_reviewed_python_sources,
};

/// The exact historical contract route whose production script is missing.
const WHITAKER: &str = include_str!("fixtures/ci_reachability/make_whitaker_contracts/Makefile");
/// A contract route that adds a second, unreviewed Python script.
const WHITAKER_EXTRA: &str =
    include_str!("fixtures/ci_reachability/make_whitaker_extra_script/Makefile");
/// A contract route that omits its required `PyYAML` pin.
const WHITAKER_NO_PYYAML: &str =
    include_str!("fixtures/ci_reachability/make_whitaker_missing_pyyaml/Makefile");
/// An opaque Python wrapper that cannot be source-identified.
const WHITAKER_OPAQUE: &str =
    include_str!("fixtures/ci_reachability/make_whitaker_opaque_script/Makefile");
/// A route that names an unreviewed script body.
const WHITAKER_UNKNOWN: &str =
    include_str!("fixtures/ci_reachability/make_whitaker_unknown_script/Makefile");
/// A contract route that does not pin its required pytest version.
const WHITAKER_UNPINNED: &str =
    include_str!("fixtures/ci_reachability/make_whitaker_unpinned_pytest/Makefile");

/// Extracts the exact executable argv after removing only the fixture's known env prefix.
fn whitaker_words(source: &str) -> Result<Vec<String>, String> {
    let recipe = source
        .lines()
        .find(|line| line.starts_with('\t'))
        .ok_or_else(|| "Whitaker fixture has no recipe".to_owned())?;
    let command = recipe.trim_start_matches('\t').replace("$(UV)", "uv");
    let tokens = make_syntax::tokenize(&command)?;
    let mut words = tokens
        .into_iter()
        .filter_map(|token| match token {
            make_syntax::Token::Word(word) => Some(word),
            _ => None,
        })
        .collect::<Vec<_>>();
    while words.first().is_some_and(|word| word.contains('=')) {
        let _ = words.remove(0);
    }
    Ok(words)
}

/// Captures the production source gap as data while preserving fixture cleanup.
fn whitaker_gap(words: &[String]) -> Result<String, String> {
    shared_tests::with_fixture(&[], |workspace| {
        classify_workspace_words(words, false, workspace)
            .err()
            .ok_or_else(|| "unreviewed Whitaker script was accepted".to_owned())
    })
}

/// Historical `pytest` and `PyYAML` syntax is lexical-only because its script is absent.
#[test]
fn make_whitaker_contracts_fixture_is_lexical_but_not_a_production_allowlist() {
    let words = whitaker_words(WHITAKER).expect("historical fixture argv must parse");
    assert_eq!(classify_words(&words, false), Ok(Route::NonSuite));
    assert_eq!(
        whitaker_gap(&words).expect("missing script gap must be explicit"),
        "Whitaker contract script is absent or unreviewed"
    );
}

/// Each source-level Whitaker mutation is detected with its own reason.
#[rstest]
#[case::make_whitaker_extra_script(
    WHITAKER_EXTRA,
    "Whitaker command has an unreviewed extra script"
)]
#[case::make_whitaker_missing_pyyaml(
    WHITAKER_NO_PYYAML,
    "Whitaker command does not pin PyYAML 6.0.3"
)]
#[case::make_whitaker_opaque_script(WHITAKER_OPAQUE, "opaque Whitaker Python script is unreviewed")]
#[case::make_whitaker_unknown_script(
    WHITAKER_UNKNOWN,
    "unresolved UV command, version, module, or script"
)]
#[case::make_whitaker_unpinned_pytest(
    WHITAKER_UNPINNED,
    "Whitaker command does not pin pytest 9.0.2"
)]
fn whitaker_fixture_mutations_have_specific_diagnostics(
    #[case] source: &str,
    #[case] expected: &str,
) {
    let baseline = whitaker_words(WHITAKER).expect("historical baseline argv must parse");
    assert_eq!(classify_words(&baseline, false), Ok(Route::NonSuite));
    let words = whitaker_words(source).expect("mutated fixture argv must parse");
    let error = classify_words(&words, false).expect_err("mutated route must fail");
    assert_eq!(error, expected);
}

/// LF and CRLF versions of all four reviewed bodies produce the same approved identities.
#[test]
fn reviewed_python_sources_accept_lf_and_crlf() -> Result<(), String> {
    shared_tests::with_fixture(&[], |workspace| {
        stage_python_bodies(workspace, LineEnding::Lf)?;
        verify_reviewed_python_sources(workspace)?;
        stage_python_bodies(workspace, LineEnding::CrLf)?;
        verify_reviewed_python_sources(workspace)
    })
}

/// Changing bytes at a known path is an explicit identity gap after a successful baseline.
#[test]
fn reviewed_python_source_content_drift_is_a_specific_gap() {
    let result = shared_tests::with_fixture(&[], |workspace| {
        stage_python_bodies(workspace, LineEnding::Lf)?;
        verify_reviewed_python_sources(workspace)?;
        let path = "scripts/check_integration_test_inventory.py";
        let original = workspace
            .directory
            .read_to_string(camino::Utf8Path::new(path))
            .map_err(|error| format!("reading staged checker: {error}"))?;
        workspace
            .directory
            .write(
                camino::Utf8Path::new(path),
                format!("{original}\n# changed body\n"),
            )
            .map_err(|error| format!("mutating staged checker: {error}"))?;
        verify_reviewed_python_sources(workspace)
    });
    let error = result.expect_err("same-path source drift must fail after baseline verification");
    assert_eq!(
        error,
        "reviewed Python source identity changed: scripts/check_integration_test_inventory.py"
    );
    assert_eq!(
        REVIEWED_PYTHON_BODIES.len(),
        4,
        "the reviewed source boundary is finite"
    );
}

/// Accepts only the complete lowercase hexadecimal form emitted by Git.
#[test]
fn git_blob_id_requires_complete_lowercase_hex() {
    let valid = "0123456789abcdef0123456789abcdef01234567";
    assert!(
        GitBlobId::parse(valid).is_ok(),
        "a complete lowercase hexadecimal ID must parse"
    );

    let invalid = [
        "0123456789ABCDEF0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef0123456g",
        "0123456789abcdef0123456789abcdef0123456",
        "0123456789abcdef0123456789abcdef012345670",
        "0123456789abcdef0123456789abcdef0123456\n",
    ];
    for text in invalid {
        assert_eq!(
            GitBlobId::parse(text),
            Err("git hash-object returned an invalid blob identifier".to_owned()),
            "invalid blob ID should fail exactly: {text:?}"
        );
    }
}
