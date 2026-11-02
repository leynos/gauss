//! Verifies Make source safety and trace binding with focused source mutations.

use rstest::rstest;

use super::super::make_routes;
use super::{
    AssignmentOperator, MakeAssignment, MakeSelector, assignment_parts, controls::condition,
    required_selectors, source_assignment, validate,
};

/// Production Make source used as the source-guard baseline.
const CURRENT_MAKE: &str = include_str!("../../Makefile");
/// A reached recipe whose ignored-error prefix must be tied to trace provenance.
const IGNORED_FAILURE: &str =
    include_str!("fixtures/ci_reachability/make_ignored_failure/Makefile");

/// Appends one focused source mutation after the known passing Makefile.
fn append_fixture(addition: &str) -> String {
    format!("{CURRENT_MAKE}\n{addition}")
}

/// Every physical recipe line is bound so later ignored-error prefixes cannot escape.
#[test]
fn make_ignored_failure_prefix_is_bound_to_its_source_recipe() {
    validate(CURRENT_MAKE).expect("production Make source must pass its baseline");
    let ordinary = IGNORED_FAILURE.replace("\t-$(CARGO)", "\t$(CARGO)");
    let baseline = append_fixture(&ordinary);
    let passing =
        make_routes::trace_for_target(&baseline, "ignored-route", "cargo test --workspace")
            .expect("ordinary fixture route must be present");
    make_routes::verify_trace(&baseline, &[passing])
        .expect("ordinary recipe should bind to source");

    let mutated = append_fixture(IGNORED_FAILURE);
    let traced = make_routes::trace_for_target(&mutated, "ignored-route", "cargo test --workspace")
        .expect("ignored fixture route must be present");
    let error = make_routes::verify_trace(&mutated, &[traced])
        .expect_err("the reached ignored-error recipe must fail closed");
    assert!(
        error.contains("ignored Make recipe prefix"),
        "unexpected trace diagnostic: {error}"
    );

    let late_baseline = append_fixture("late-route:\n\t@echo prepare\n\t$(CARGO) test --workspace");
    validate(&late_baseline).expect("ordinary multi-line recipe must pass source checks");
    let late_baseline_trace = make_routes::trace_for_target(
        &late_baseline,
        "late-route",
        "echo prepare\ncargo test --workspace",
    )
    .expect("multi-line recipe target must be present");
    make_routes::verify_trace(&late_baseline, &[late_baseline_trace])
        .expect("later ordinary recipe line must bind to the target");

    let late_mutant = late_baseline.replace(
        "\t$(CARGO) test --workspace",
        "\t-$(CARGO) test --workspace",
    );
    validate(&late_mutant)
        .expect("ignored-error prefixes require trace binding rather than source rejection");
    let late_mutant_trace = make_routes::trace_for_target(
        &late_mutant,
        "late-route",
        "echo prepare\ncargo test --workspace",
    )
    .expect("mutated multi-line target must remain present");
    let late_error = make_routes::verify_trace(&late_mutant, &[late_mutant_trace])
        .expect_err("a later ignored-error recipe line must fail closed");
    assert_eq!(
        late_error,
        format!(
            "ignored Make recipe prefix at line {}",
            late_mutant.lines().count()
        ),
    );
}

/// Each named input retains both readers' full typed result and any operator-spelling proof.
#[rstest]
#[case::conditional_operator(
    "NAME?= value",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Conditional,
        value: "value",
    }),
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Conditional,
        value: "value",
    }),
    Some((AssignmentOperator::Conditional, "?=")),
)]
#[case::immediate_operator(
    "NAME:= value",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Immediate,
        value: "value",
    }),
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Immediate,
        value: "value",
    }),
    Some((AssignmentOperator::Immediate, ":=")),
)]
#[case::append_operator(
    "NAME+= value",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Append,
        value: "value",
    }),
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Append,
        value: "value",
    }),
    Some((AssignmentOperator::Append, "+=")),
)]
#[case::shell_operator(
    "NAME!= value",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Shell,
        value: "value",
    }),
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Shell,
        value: "value",
    }),
    Some((AssignmentOperator::Shell, "!=")),
)]
#[case::recursive_operator(
    "NAME= value",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Recursive,
        value: "value",
    }),
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Recursive,
        value: "value",
    }),
    Some((AssignmentOperator::Recursive, "=")),
)]
#[case::exported_assignment(
    "export NAME := value",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Immediate,
        value: "value",
    }),
    Some(MakeAssignment {
        name: "export NAME",
        operator: AssignmentOperator::Immediate,
        value: "value",
    }),
    None,
)]
#[case::target_specific_assignment(
    "target: NAME ?= value",
    None,
    Some(MakeAssignment {
        name: "target: NAME",
        operator: AssignmentOperator::Conditional,
        value: "value",
    }),
    None,
)]
#[case::invalid_simple_name(
    "bad name = value",
    None,
    Some(MakeAssignment {
        name: "bad name",
        operator: AssignmentOperator::Recursive,
        value: "value",
    }),
    None,
)]
#[case::operator_in_value(
    "NAME = before?=after",
    Some(MakeAssignment {
        name: "NAME",
        operator: AssignmentOperator::Recursive,
        value: "before?=after",
    }),
    Some(MakeAssignment {
        name: "NAME = before",
        operator: AssignmentOperator::Conditional,
        value: "after",
    }),
    None,
)]
fn assignment_views_preserve_source_parser_policies(
    #[case] source: &'static str,
    #[case] expected_parts: Option<MakeAssignment<'static>>,
    #[case] expected_source: Option<MakeAssignment<'static>>,
    #[case] spelling: Option<(AssignmentOperator, &'static str)>,
) {
    assert_eq!(assignment_parts(source), expected_parts);
    assert_eq!(source_assignment(source), expected_source);
    if let Some((operator, expected)) = spelling {
        assert_eq!(operator.spelling(), expected);
    }
}

/// Each supported literal retains all three original field assertions and its record diagnostic.
#[rstest]
#[case::host_selector(
    "ifeq ($(BUILD_HOST_OS),Linux)",
    (MakeSelector::BuildHostOs, "BUILD_HOST_OS", "Linux"),
    "supported host selector must produce a condition record",
)]
#[case::mode_selector(
    "ifneq ($(MODE),coverage)",
    (MakeSelector::Mode, "MODE", "coverage"),
    "supported mode selector must produce a condition record",
)]
fn condition_records_preserve_literal_fields(
    #[case] source: &str,
    #[case] expected: (MakeSelector, &str, &str),
    #[case] record_reason: &str,
) {
    let parsed = condition(source).expect(record_reason);
    let (expected_selector, expected_name, expected_comparison) = expected;
    assert_eq!(parsed.selector, expected_selector);
    assert_eq!(parsed.selector.name(), expected_name);
    assert_eq!(parsed.comparison, expected_comparison);
}

/// Each invalid condition must fail for its own reason after a complete supported baseline.
#[rstest]
#[case::unknown_before_empty(
    "ifeq ($(UNKNOWN),)",
    "tool-dependent or unknown Make selector: UNKNOWN"
)]
#[case::computed_before_unknown(
    "ifeq ($(UNKNOWN),$(MODE))",
    "computed Make condition value: ifeq ($(UNKNOWN),$(MODE))"
)]
fn condition_records_preserve_error_precedence(#[case] source: &str, #[case] expected: &str) {
    let baseline = condition("ifeq ($(BUILD_HOST_OS),Linux)")
        .expect("supported host selector must produce a condition record");
    assert_eq!(baseline.selector, MakeSelector::BuildHostOs);
    assert_eq!(baseline.selector.name(), "BUILD_HOST_OS");
    assert_eq!(baseline.comparison, "Linux");
    assert_eq!(condition(source).err().as_deref(), Some(expected));
}

/// Typed selector iteration retains the old source-name ordering independently of input order.
#[test]
fn condition_selector_set_preserves_name_order() {
    let selectors = required_selectors(
        "ifeq ($(MODE),coverage)\nendif\nifeq ($(BUILD_HOST_OS),Linux)\nendif\n",
    )
    .expect("supported selectors must form a typed set");
    assert_eq!(
        selectors.into_iter().collect::<Vec<_>>(),
        vec![MakeSelector::BuildHostOs, MakeSelector::Mode],
    );
}
