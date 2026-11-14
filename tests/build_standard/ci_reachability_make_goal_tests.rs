//! Specifies the literal goal grammar closed by `MakeGoal`.

use rstest::rstest;

use super::{CURRENT_MAKE, Host, MakeCase, evaluate_case, make_routes::MakeGoal};

/// Goals carrying computed syntax or no text never reach GNU Make.
#[rstest]
#[case::make_expression("$(MODE)")]
#[case::shell_function("$(shell true)")]
#[case::sequence("test;true")]
#[case::backtick("`id`")]
#[case::empty("")]
fn nonliteral_goals_are_rejected_by_the_parser(#[case] goal: &str) {
    let error = MakeGoal::parse(goal).expect_err("a non-literal goal must be refused");
    assert_eq!(error, format!("unresolved Make option or goal: {goal}"));
}

/// A literal goal parses and hands GNU Make exactly the reviewed text.
#[rstest]
#[case::target("test")]
#[case::path("crates/gauss-core")]
fn literal_goals_round_trip(#[case] goal: &str) {
    let parsed = MakeGoal::parse(goal).expect("a literal goal must parse");
    assert_eq!(parsed.as_str(), goal);
}

/// A computed goal fails before the fixture exists, so no Make run can start.
#[test]
fn evaluation_gap_precedes_the_fixture() {
    let error = evaluate_case(&MakeCase {
        source: CURRENT_MAKE,
        goal: "$(shell true)",
        mode: None,
        host: Host::Linux,
    })
    .expect_err("a computed goal must not reach the dry run");
    assert_eq!(error, "unresolved Make option or goal: $(shell true)");
}
