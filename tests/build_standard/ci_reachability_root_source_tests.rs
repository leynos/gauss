//! Exercises fail-closed Make source parsing against unsafe source mutations.

use rstest::rstest;

use super::super::make_source;

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
