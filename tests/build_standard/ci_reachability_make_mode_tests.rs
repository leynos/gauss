//! Specifies the Make argument grammar enforced before GNU Make starts.

use rstest::rstest;

use super::{
    CURRENT_MAKE,
    Host,
    MakeCase,
    ROUTES,
    Route,
    append_fixture,
    classify_workflow_command,
    evaluate_case,
};

/// The direct evaluator rejects MODE values the workflow adapter already refuses.
#[rstest]
#[case::empty("")]
#[case::whitespace(" ")]
#[case::trailing_space("coverage ")]
#[case::make_expression("$(MODE)")]
#[case::shell_metacharacter("release;true")]
fn direct_evaluation_rejects_nonliteral_mode(#[case] mode: &str) {
    let error = evaluate_case(&MakeCase {
        source: CURRENT_MAKE,
        goal: "test",
        mode: Some(mode),
        host: Host::Linux,
    })
    .expect_err("a non-literal MODE must be rejected before GNU Make starts");
    assert!(
        error.contains("MODE has an unresolved value"),
        "unexpected MODE diagnostic: {error}"
    );
}

/// The invariant also applies when a source MODE condition would otherwise be satisfied.
#[rstest]
#[case::empty("")]
#[case::make_expression("$(MODE)")]
fn conditional_source_rejects_nonliteral_mode(#[case] mode: &str) {
    let source = append_fixture(ROUTES);
    let error = evaluate_case(&MakeCase {
        source: &source,
        goal: "mode-suite",
        mode: Some(mode),
        host: Host::Linux,
    })
    .expect_err("a non-literal MODE must not satisfy the source selector");
    assert!(
        error.contains("MODE has an unresolved value"),
        "unexpected MODE diagnostic: {error}"
    );
}

/// Workflow Make spellings outside the literal grammar fail with one specific gap.
#[rstest]
#[case::make_option("make -j2 test", "unresolved Make option or goal: -j2")]
#[case::missing_goal("make", "Make invocation has no explicit goal")]
#[case::duplicate_mode(
    "make MODE=coverage MODE=release test",
    "unresolved Make command-line assignment: MODE"
)]
#[case::empty_mode("make MODE= test", "Make selector MODE has an unresolved value: ")]
#[case::quoted_metacharacter_goal("make 'test;true'", "unresolved Make option or goal: test;true")]
fn workflow_make_argument_gaps_are_specific(#[case] command: &str, #[case] expected: &str) {
    let error = classify_workflow_command(command)
        .expect_err("a Make command outside the literal grammar must not be evaluated");
    assert_eq!(error, expected);
}

/// One literal MODE assignment is admitted and routes to the suite.
#[test]
fn workflow_make_literal_mode_assignment_routes_to_the_suite() {
    assert_eq!(
        classify_workflow_command("make MODE=release test")
            .expect("a literal MODE assignment must remain evaluable"),
        Route::Suite
    );
}
