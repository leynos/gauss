//! Proves selected Make routes and every Cargo invocation row independently.

use rstest::rstest;

use super::{
    Host, MakeCase, ROUTES, Route, SPELLING_SUITE, append_fixture, cargo_rows, evaluate_case,
    evaluate_current, has_words, make_routes, make_tools, tests,
};

/// Lists every expected Cargo argv row, in GNU Make's printed order, for a supported profile.
/// For example, `default` requires four rows while an unknown profile returns an error.
pub(super) fn expected_cargo_rows(profile: &str) -> Result<Vec<Vec<&'static str>>, String> {
    let cargo = make_tools::PROBE_CARGO;
    match profile {
        "default" => Ok(vec![
            vec![cargo, "nextest", "--version"],
            vec![
                cargo,
                "nextest",
                "run",
                "--profile",
                "default",
                "--workspace",
                "--all-targets",
                "--all-features",
            ],
            vec![
                cargo,
                "test",
                "--workspace",
                "--all-targets",
                "--all-features",
            ],
            vec![cargo, "test", "--workspace", "--doc", "--all-features"],
        ]),
        "ci" => Ok(vec![vec![
            cargo,
            "nextest",
            "run",
            "--profile",
            "ci",
            "--workspace",
            "--all-targets",
            "--all-features",
        ]]),
        other => Err(format!(
            "unsupported Cargo profile in route contract: {other}"
        )),
    }
}

/// Requires full Cargo argv rows in expected order, without truncating extra invocations.
/// For example, expected `cargo test` rejects an actual row with an extra `--help` switch.
pub(super) fn require_ordered_cargo_rows(
    actual: &[&super::Invocation],
    expected: &[Vec<&str>],
) -> Result<(), String> {
    if actual.len() != expected.len() {
        return Err(format!(
            "Cargo row count differs: expected {}, received {}",
            expected.len(),
            actual.len()
        ));
    }
    for (position, (invocation, expected_words)) in actual.iter().zip(expected).enumerate() {
        let matches = invocation.words.len() == expected_words.len()
            && invocation
                .words
                .iter()
                .zip(expected_words)
                .all(|(word, expected_word)| word.as_str() == *expected_word);
        if !matches {
            return Err(format!(
                "Cargo row {} differs from its complete expected argv",
                position + 1
            ));
        }
    }
    Ok(())
}

/// The Make `test` goal classifies every printed probe, both suite branches, and its doctest.
#[test]
fn make_test_inspects_probe_nextest_fallback_and_doctest_rows() {
    let evaluation =
        evaluate_current("test", Host::Linux).expect("the production test route must be evaluable");
    assert_eq!(evaluation.route, Route::Suite);
    let rows = cargo_rows(&evaluation);
    let expected = expected_cargo_rows("default").expect("default test rows must be specified");
    require_ordered_cargo_rows(&rows, &expected)
        .expect("probe, both suite branches and doctest must match their complete argv");
}

/// A later unknown prerequisite cannot be skipped after an earlier suite recipe.
#[test]
fn make_inspects_unknown_prerequisite_after_suite() {
    assert_eq!(
        evaluate_current("test", Host::Linux)
            .expect("suite baseline must resolve")
            .route,
        Route::Suite
    );
    let source = format!(
        "{}\nsuite-then-unknown: test later-unknown\nlater-unknown:\n\tunreviewed-tool command\n",
        append_fixture(ROUTES)
    );
    let error = evaluate_case(&MakeCase {
        source: &source,
        goal: "suite-then-unknown",
        mode: Some("coverage"),
        host: Host::Linux,
    })
    .expect_err("an unknown command after the suite must remain a reader gap");
    assert!(
        error.contains("unresolved executable route: unreviewed-tool"),
        "{error}"
    );
}

/// The direct workflow adapter delegates the real `make test` command to evaluated GNU Make.
#[test]
fn workflow_make_command_reaches_test_prerequisites() {
    let route = tests::with_fixture(&[], |workspace| {
        make_tools::stage_python_bodies(workspace, make_tools::LineEnding::Lf)?;
        make_routes::classify("make test", workspace, Host::Linux)
    })
    .expect("workflow Make command must resolve its prerequisites");
    assert_eq!(route, Route::Suite);
}

/// CI and quick goals each retain a separately inspected Nextest invocation.
#[rstest]
#[case::make_test_ci("test-ci", "ci", &["--workspace", "--all-targets", "--all-features"])]
#[case::make_test_quick("test-quick", "default", &["--lib", "--workspace", "--all-targets", "--all-features"])]
fn make_suite_leaf_has_its_own_cargo_row(
    #[case] goal: &str,
    #[case] profile: &str,
    #[case] flags: &[&str],
) {
    let evaluation =
        evaluate_current(goal, Host::Linux).expect("selected Make suite goal must resolve");
    assert_eq!(evaluation.route, Route::Suite);
    let rows = cargo_rows(&evaluation);
    assert_eq!(rows.len(), 1, "leaf suite goal has one Cargo invocation");
    let mut expected = vec![
        make_tools::PROBE_CARGO,
        "nextest",
        "run",
        "--profile",
        profile,
    ];
    expected.extend_from_slice(flags);
    require_ordered_cargo_rows(&rows, &[expected])
        .expect("leaf Cargo invocation must match its complete expected argv");
}

/// Documentation and lint goals prove their rustdoc and Clippy rows independently.
#[test]
fn make_lint_inspects_rustdoc_clippy_and_whitaker_rows() {
    let evaluation = evaluate_current("lint", Host::Linux).expect("lint goal must resolve");
    assert_eq!(evaluation.route, Route::NonSuite);
    let rows = cargo_rows(&evaluation);
    let cargo = make_tools::PROBE_CARGO;
    assert_eq!(rows.len(), 2, "rustdoc and Clippy each need a Cargo proof");
    assert!(
        rows.iter()
            .any(|row| has_words(&row.words, &[cargo, "doc", "--workspace", "--no-deps"])),
        "missing rustdoc invocation"
    );
    assert!(
        rows.iter().any(|row| has_words(
            &row.words,
            &[
                cargo,
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ]
        )),
        "missing Clippy invocation"
    );
    assert!(
        evaluation.invocations.iter().any(|row| {
            row.words
                .first()
                .is_some_and(|word| word == make_tools::PROBE_WHITAKER)
        }),
        "missing non-Cargo Whitaker gate row"
    );
}

/// The current Cargo pattern recipe binds its trace to its supported source target.
#[test]
fn make_cargo_pattern_recipe_has_a_bound_trace() {
    let evaluation = evaluate_current("target/debug/libgauss.rlib", Host::Linux)
        .expect("the current Cargo pattern goal must resolve through its recipe");
    assert_eq!(evaluation.route, Route::NonSuite);
    assert!(
        evaluation
            .blocks
            .iter()
            .any(|block| block.target == "target/debug/libgauss.rlib"),
        "the Cargo pattern target must be present in the trace"
    );
    assert!(
        cargo_rows(&evaluation)
            .iter()
            .any(|row| { has_words(&row.words, &[make_tools::PROBE_CARGO, "build"]) }),
        "the traced pattern recipe must retain its Cargo build invocation"
    );
}

/// An alias or supported explicit pattern prerequisite reaches the actual test recipe.
#[rstest]
#[case::make_alias("verify-stable")]
#[case::make_pattern_route("suite-smoke")]
fn indirect_make_prerequisite_reaches_the_test_suite(#[case] goal: &str) {
    let source = append_fixture(ROUTES);
    let evaluation = evaluate_case(&MakeCase {
        source: &source,
        goal,
        mode: Some("coverage"),
        host: Host::Linux,
    })
    .expect("alias or explicit pattern route must be evaluated by GNU Make");
    assert_eq!(
        evaluation.route,
        Route::Suite,
        "{goal} must retain its test prerequisite"
    );
    let expected = expected_cargo_rows("default").expect("coverage rows must be specified");
    require_ordered_cargo_rows(&cargo_rows(&evaluation), &expected)
        .expect("each reached Cargo command must match its complete expected argv");
}

/// Expected profile and diagnostics for one explicit suite-selector scenario.
struct SuiteBranchProof {
    /// Profile whose complete ordered Cargo rows must be selected.
    profile: &'static str,
    /// Context if GNU Make cannot evaluate the selected branch.
    setup_reason: &'static str,
    /// Context if the expected profile has no Cargo-row specification.
    profile_reason: &'static str,
    /// Context if any selected Cargo row differs from its specification.
    rows_reason: &'static str,
}

/// MODE and host selectors retain their distinct branches and complete Cargo-row proofs.
#[rstest]
#[case::make_mode_coverage(
    "mode-suite", "coverage", Host::Linux,
    &SuiteBranchProof {
        profile: "default",
        setup_reason: "explicit MODE branch must be selected by Make",
        profile_reason: "the selected profile must have row proofs",
        rows_reason: "every explicit MODE branch must match all ordered Cargo argv rows",
    }
)]
#[case::make_mode_release(
    "mode-suite", "release", Host::Linux,
    &SuiteBranchProof {
        profile: "ci",
        setup_reason: "explicit MODE branch must be selected by Make",
        profile_reason: "the selected profile must have row proofs",
        rows_reason: "every explicit MODE branch must match all ordered Cargo argv rows",
    }
)]
#[case::make_host_linux(
    "host-suite", "coverage", Host::Linux,
    &SuiteBranchProof {
        profile: "default",
        setup_reason: "known host branch must be selected by GNU Make",
        profile_reason: "the host branch profile must be covered",
        rows_reason: "every host branch must match its complete ordered Cargo argv rows",
    }
)]
#[case::make_host_darwin(
    "host-suite", "coverage", Host::Darwin,
    &SuiteBranchProof {
        profile: "ci",
        setup_reason: "known host branch must be selected by GNU Make",
        profile_reason: "the host branch profile must be covered",
        rows_reason: "every host branch must match its complete ordered Cargo argv rows",
    }
)]
fn explicit_selector_keeps_complete_suite_routes(
    #[case] goal: &str,
    #[case] mode: &str,
    #[case] host: Host,
    #[case] proof: &SuiteBranchProof,
) {
    let source = append_fixture(ROUTES);
    let evaluation = evaluate_case(&MakeCase {
        source: &source,
        goal,
        mode: Some(mode),
        host,
    })
    .expect(proof.setup_reason);
    let rows = cargo_rows(&evaluation);
    assert_eq!(evaluation.route, Route::Suite);
    let expected = expected_cargo_rows(proof.profile).expect(proof.profile_reason);
    require_ordered_cargo_rows(&rows, &expected).expect(proof.rows_reason);
}

/// An explicit selector baseline passes before the same conditional is tried without MODE.
#[test]
fn make_mode_selector_missing_is_a_specific_gap() {
    let source = append_fixture(ROUTES);
    let baseline = evaluate_case(&MakeCase {
        source: &source,
        goal: "mode-suite",
        mode: Some("coverage"),
        host: Host::Linux,
    })
    .expect("explicit MODE baseline must remain evaluable");
    let expected = expected_cargo_rows("default").expect("coverage rows must be specified");
    require_ordered_cargo_rows(&cargo_rows(&baseline), &expected)
        .expect("explicit MODE baseline must prove every selected Cargo row");

    let error = evaluate_case(&MakeCase {
        source: &source,
        goal: "mode-suite",
        mode: None,
        host: Host::Linux,
    })
    .expect_err("a missing conditional selector must remain a reader gap");
    assert_eq!(error, "Make selector MODE has no explicit workflow value");
}

/// A spelling target that gains a test prerequisite cannot remain classified as non-suite.
#[test]
fn spelling_goal_with_new_suite_prerequisite_is_reclassified() {
    assert_eq!(
        SPELLING_SUITE, "spelling: test\n",
        "mutation adds one literal prerequisite"
    );
    let baseline = evaluate_current("spelling", Host::Linux)
        .expect("the unmodified spelling goal must be evaluable");
    assert_eq!(
        baseline.route,
        Route::NonSuite,
        "spelling starts without a suite prerequisite"
    );
    assert!(
        cargo_rows(&baseline).is_empty(),
        "spelling baseline must emit no Cargo rows"
    );

    let source = append_fixture(SPELLING_SUITE);
    let evaluation = evaluate_case(&MakeCase {
        source: &source,
        goal: "spelling",
        mode: None,
        host: Host::Linux,
    })
    .expect("spelling mutation must be evaluated through GNU Make");
    assert_eq!(evaluation.route, Route::Suite);
    let expected = expected_cargo_rows("default").expect("the test prerequisite rows must exist");
    require_ordered_cargo_rows(&cargo_rows(&evaluation), &expected)
        .expect("the new prerequisite must emit every complete Cargo row");
}
