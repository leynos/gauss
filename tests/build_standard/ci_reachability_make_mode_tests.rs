//! Specifies that direct Make evaluation enforces the MODE lexical invariant.

use rstest::rstest;

use super::{CURRENT_MAKE, Host, MakeCase, ROUTES, append_fixture, evaluate_case};

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
