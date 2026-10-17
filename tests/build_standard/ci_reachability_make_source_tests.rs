//! Verifies Make source safety and trace binding with focused source mutations.

use super::super::make_routes;
use super::validate;

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
