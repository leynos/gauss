//! Exercises fail-closed Make source mutations and source-bound GNU Make trace evidence.

use rstest::rstest;

use super::super::{Route, make_output, make_source};

/// The current source baseline used before each Make safety mutation.
const CURRENT_MAKE_SOURCE: &str = include_str!("../../Makefile");
/// An unreviewed parse-time shell function that must fail closed.
const INLINE_EXECUTION: &str =
    include_str!("fixtures/ci_reachability/make_inline_execution/Makefile");
/// A computed target that could hide a Makefile-remaking rule.
const COMPUTED_TARGET: &str =
    include_str!("fixtures/ci_reachability/make_computed_target/Makefile");
/// A global `.IGNORE` directive that masks recipe failures.
const IGNORE: &str = include_str!("fixtures/ci_reachability/make_ignore/Makefile");
/// A global `.ONESHELL` directive that changes shell failure propagation.
const ONESHELL: &str = include_str!("fixtures/ci_reachability/make_oneshell/Makefile");
/// A recursive Make recipe prefix that can escape the dry-run.
const RECURSIVE: &str = include_str!("fixtures/ci_reachability/make_recursive/Makefile");
/// An uninspected include that could add executable Make source.
const UNINSPECTED_INCLUDE: &str =
    include_str!("fixtures/ci_reachability/make_uninspected_include/Makefile");
/// A tool-dependent condition that cannot be resolved from the workflow host.
const UNSUPPORTED_SELECTOR: &str =
    include_str!("fixtures/ci_reachability/make_unsupported_selector/Makefile");

/// Appends one source mutation to the reviewed Make baseline.
fn append_make_mutation(addition: &str) -> String {
    format!("{CURRENT_MAKE_SOURCE}\n{addition}")
}

/// Returns one exact Make source gap after requiring the current baseline to pass.
fn syntax_gap(addition: &str) -> Result<String, String> {
    make_source::validate(CURRENT_MAKE_SOURCE)
        .map_err(|error| format!("current Makefile failed its source baseline: {error}"))?;
    make_source::validate(&append_make_mutation(addition))
        .err()
        .ok_or_else(|| "Make source mutation did not produce its expected gap".to_owned())
}

/// Every unsupported Make source form is detected from a passing baseline and one fixture delta.
#[rstest]
#[case::make_computed_target(
    COMPUTED_TARGET,
    "computed Make target cannot be proved safe: $(COMPUTED_GOAL)"
)]
#[case::make_ignore(IGNORE, "unprovable Make global directive: .IGNORE:")]
#[case::make_spaced_ignore(
    "target .IGNORE :\n\t$(CARGO) test --workspace",
    "unprovable Make global directive: target .IGNORE :"
)]
#[case::make_inline_execution(INLINE_EXECUTION, "unreviewed Make execution function: shell")]
#[case::make_oneshell(ONESHELL, "unprovable Make global directive: .ONESHELL:")]
#[case::make_recursive(RECURSIVE, "Make + recipe can execute during dry-run")]
#[case::make_uninspected_include(UNINSPECTED_INCLUDE, "uninspected Make include")]
#[case::make_unsupported_selector(
    UNSUPPORTED_SELECTOR,
    "tool-dependent or unknown Make selector: CARGO"
)]
#[case::make_load("load extension.so", "uninspected Make load directive")]
#[case::make_minus_load("-load extension.so", "uninspected Make load directive")]
#[case::make_recipe_prefix_assignment(
    ".RECIPEPREFIX := >",
    "unprovable Make global directive: .RECIPEPREFIX := >"
)]
#[case::make_shell_flags_assignment(
    ".SHELLFLAGS := -c",
    "unprovable Make global directive: .SHELLFLAGS := -c"
)]
fn unsafe_make_source_mutations_are_identified(#[case] addition: &str, #[case] expected: &str) {
    assert_eq!(
        syntax_gap(addition).expect("specific Make source gap must be reported"),
        expected
    );
}

/// Source-defined fallback, suffix, and target definitions cannot regain Makefile remaking.
#[rstest]
#[case::make_default(
    ".DEFAULT:\n\t$(CARGO) test --workspace\n",
    "Make .DEFAULT can remake the selected Makefile"
)]
#[case::make_suffix(
    ".in.out:\n\t$(CARGO) test --workspace\n",
    "source-defined Make suffix rule is unprovable: .in.out"
)]
#[case::make_pattern_remake(
    "Make%:\n\t$(CARGO) test --workspace\n",
    "Make rule could remake its Makefile: Make%"
)]
#[case::make_duplicate_target(
    "TARGET ?= libgauss.rlib",
    "Make TARGET definition is missing or not unique"
)]
#[case::make_export_target(
    "export TARGET ?= libgauss.rlib",
    "Make TARGET definition is not canonical"
)]
#[case::make_conditional_target(
    "ifeq ($(MODE),coverage)\nTARGET ?= libgauss.rlib\nendif",
    "conditional Make TARGET definition is unprovable"
)]
fn source_rules_that_could_remake_makefile_are_gaps(
    #[case] addition: &str,
    #[case] expected: &str,
) {
    assert_eq!(
        syntax_gap(addition).expect("Makefile source hazard must be reported"),
        expected
    );
}

/// A target with no prerequisites, matching GNU Make 4.3's missing-target trace.
const MISSING_TARGET_SOURCE: &str = ".PHONY: test\ntest:\n\tcargo test --workspace\n";
/// A target whose phony prerequisite gives the changed-prerequisite trace its reason.
const PREREQUISITE_SOURCE: &str =
    ".PHONY: test force\ntest: force\n\tcargo test --workspace\nforce:\n";

/// Resolves a synthetic trace through the real source-binding and command classifier.
/// For example, an unbound source line returns its existing location diagnostic.
fn trace_fixture(header: &str, source: &str) -> Result<make_output::Evaluation, String> {
    let output = format!("{header}\ncargo test --workspace\n");
    make_output::classify_traced(&output, source, false, |invocation, controlled| {
        make_output::resolve(invocation, controlled, None)
    })
}

/// Both source-supported GNU Make 4.3 headers retain the complete suite evidence.
#[rstest]
#[case::make_43_missing_target("Makefile:3: target 'test' does not exist", MISSING_TARGET_SOURCE)]
#[case::make_43_prerequisites(
    "Makefile:3: update target 'test' due to: force",
    PREREQUISITE_SOURCE
)]
fn supported_trace_header_formats_keep_bound_suite_evidence(
    #[case] header: &str,
    #[case] source: &str,
) {
    let evidence = trace_fixture(header, source).expect("supported trace must remain source-bound");
    assert_eq!(evidence.route, Route::Suite);
    assert_eq!(
        evidence.blocks,
        vec![make_output::TraceBlock {
            file: "Makefile".to_owned(),
            line: 3,
            target: "test".to_owned(),
            command: "cargo test --workspace".to_owned(),
        }],
    );
    let arguments: Vec<_> = evidence
        .invocations
        .into_iter()
        .map(|item| item.words)
        .collect();
    assert_eq!(arguments, vec![vec!["cargo", "test", "--workspace"]]);
}

/// Malformed and foreign trace headers fail before any source or command evidence is omitted.
#[test]
fn trace_header_forms_reject_malformed_or_unbound_evidence() {
    let baseline = trace_fixture(
        "Makefile:3: update target 'test' due to: force",
        PREREQUISITE_SOURCE,
    )
    .expect("prerequisite trace baseline must pass before negative controls");
    assert_eq!(baseline.route, Route::Suite);
    let arguments: Vec<_> = baseline
        .invocations
        .into_iter()
        .map(|item| item.words)
        .collect();
    assert_eq!(arguments, vec![vec!["cargo", "test", "--workspace"]]);

    let unbound = "GNU Make trace target or prerequisite is unbound";
    let empty = "GNU Make trace has an empty location, target, or reason";
    let numeric = "GNU Make trace line is not numeric";
    let foreign = "GNU Make trace names an unreviewed source file";
    let failures = [
        ("Makefile:3: target 'test' does not exist trailing", unbound),
        ("Makefile:3: target 'test' does not exists", unbound),
        ("Makefile:3: target '' does not exist", empty),
        ("Makefile:0: target 'test' does not exist", empty),
        ("Makefile:bad: target 'test' does not exist", numeric),
        ("Makefile:bad: target 'broken", numeric),
        ("Makefile:bad: update target 'broken", numeric),
        ("Makefile:3: update target 'test' due to: ", empty),
        ("Makefile:3: update target 'test' due to:   ", empty),
        (
            "Makefile:3: rebuilding test",
            "unrecognized GNU Make trace header",
        ),
        ("other.mk:3: update target 'test' due to: force", foreign),
        ("other.mk:3: target 'test' does not exist", foreign),
        (
            "Makefile:99: target 'test' does not exist",
            "GNU Make trace line is outside Makefile: 99",
        ),
        (
            "Makefile:3: target 'other' does not exist",
            "GNU Make trace target is not a literal source target: other",
        ),
        (
            concat!(
                "Makefile:3: target 'test' does not exist\n",
                "cargo test --workspace\n",
                "other.mk:3: target 'test' does not exist",
            ),
            foreign,
        ),
    ];
    for (header, expected) in failures {
        assert_eq!(
            trace_fixture(header, MISSING_TARGET_SOURCE)
                .err()
                .as_deref(),
            Some(expected),
            "wrong source-bound trace diagnostic for {header:?}",
        );
    }
}
