//! Regression coverage for step returns spelled through a `Result` alias.
//!
//! `rstest-bdd` 0.6.0 resolves an unhinted non-unit step return by its
//! **concrete type** at runtime rather than by the spelling in the signature.
//! Before that change, a step whose return type was written as a local alias
//! of `Result<T, E>` (here `test_support::TestSupportResult`) was classified as
//! an opaque *value*: the whole `Result` was boxed as a payload and any `Err`
//! was silently discarded, so a failing step still produced a green scenario.
//!
//! Both directions of that contract are pinned below. The first scenario
//! proves the alias injects the `Ok` payload: the step returns a sentinel
//! through the alias and a later step reads that sentinel back out of fixture
//! injection, so neither a discarded nor a boxed payload can satisfy it. The
//! second, run under `#[should_panic]`, proves a step returning `Err` through
//! the same alias actually fails the scenario. Under the old classification
//! the second test would have failed — the discarded `Err` would leave the
//! scenario green and leave `#[should_panic]` unsatisfied.

use gauss_core::model::Document;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct AliasWorld {
    doc: Document,
    /// Set by the failing step to prove it ran to completion before erroring.
    reached_failure: bool,
}

#[fixture]
fn world() -> AliasWorld {
    AliasWorld::default()
}

/// A payload type that exists only to make step-return injection observable.
///
/// Deliberately *not* carried on [`AliasWorld`]: if the value reached the
/// assertion through a world field, the assertion would still pass when the
/// framework never injected the step's `Ok` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AliasSentinel(&'static str);

/// The value the aliased `Ok` step returns, and therefore the value the
/// asserting step must observe.
const RETURNED_SENTINEL: &str = "returned through the alias";

/// The value the asserting step must *not* observe. Failed injection leaves
/// the fixture at this default, so the assertion cannot pass by accident.
const FIXTURE_SENTINEL: &str = "fixture default";

/// The single fixture of type [`AliasSentinel`], giving the framework's
/// `TypeId` scan exactly one candidate to override.
#[fixture]
fn alias_sentinel() -> AliasSentinel {
    AliasSentinel(FIXTURE_SENTINEL)
}

#[given("an empty document for alias classification")]
fn empty_document_for_alias_classification(world: &mut AliasWorld) {
    world.doc = Document::new();
    world.reached_failure = false;
}

/// A step that succeeds while returning through the `Result` alias.
///
/// The `Ok` payload must be unwrapped and injected as the step's value; the
/// alias must not be treated as an opaque payload. Returning a non-unit
/// sentinel is what makes that observable: `Ok(())` would carry nothing a
/// later step could distinguish from a discarded value.
#[when("a step returns a sentinel through the result alias")]
fn step_returns_sentinel_through_result_alias(
    world: &mut AliasWorld,
) -> TestSupportResult<AliasSentinel> {
    if !world.doc.visually_eq(&Document::new()) {
        return Err(TestSupportError::expectation(
            "expected a freshly constructed document to match an empty document",
        ));
    }
    Ok(AliasSentinel(RETURNED_SENTINEL))
}

/// Reads the sentinel back from fixture injection.
///
/// The fixture is always resolvable — absent an override it falls back to the
/// `#[fixture]` default — so a failed injection reaches this assertion's own
/// diagnostic rather than an unrelated resolution error.
#[then("the returned sentinel is injected as the step value")]
fn returned_sentinel_is_injected(alias_sentinel: AliasSentinel) -> TestSupportResult<()> {
    if alias_sentinel != AliasSentinel(RETURNED_SENTINEL) {
        return Err(TestSupportError::expectation(format!(
            "the Ok payload was not injected; the fixture still holds {alias_sentinel:?}"
        )));
    }
    Ok(())
}

/// A step that fails while returning through the `Result` alias.
///
/// The `Err` must propagate and fail the enclosing scenario; it must not be
/// boxed as a successful payload.
#[when("a step returns Err through the result alias")]
fn step_returns_err_through_result_alias(world: &mut AliasWorld) -> TestSupportResult<()> {
    world.reached_failure = true;
    Err(TestSupportError::expectation(
        "deliberate failure to prove the result alias propagates Err",
    ))
}

#[then("the failing step is not reached")]
fn failing_step_is_not_reached(world: &AliasWorld) -> TestSupportResult<()> {
    if world.reached_failure {
        return Err(TestSupportError::expectation(
            "the failing step should not have run in the success scenario",
        ));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/result_alias_classification.feature",
    name = "A step returning Ok through the result alias succeeds"
)]
fn step_returning_ok_through_result_alias_succeeds(
    world: AliasWorld,
    alias_sentinel: AliasSentinel,
) {
    // Both fixtures must be declared here, not only on the steps: the scenario
    // function is what seeds them into `StepContext` for the steps to resolve.
    let _ = (world, alias_sentinel);
}

// `#[should_panic]` is the assertion here: it fails if the scenario passes.
//
// The expected substring is the failing step's own text, deliberately *not* the
// template's fixed preamble. `rstest-bdd` renders this diagnostic from the
// `execution-error-handler-failed` template under a fixed `en-US` locale (the
// loader does not read the environment, so the message is not
// locale-configurable), and it wraps every interpolated field in U+2068/U+2069
// isolate characters — too brittle to match byte-exactly, and a preamble-only
// assertion such as `"Step failed at index"` would also be satisfied by *any*
// failing step, including a mis-attributed one. Pinning the step text instead
// proves the scenario failed at the aliased `When` step, i.e. that the `Err`
// returned through the alias propagated.
#[scenario(
    path = "tests/features/result_alias_classification.feature",
    name = "A step returning Err through the result alias fails the scenario"
)]
#[should_panic(expected = "a step returns Err through the result alias")]
fn step_returning_err_through_result_alias_fails_scenario(
    world: AliasWorld,
    alias_sentinel: AliasSentinel,
) {
    // The `Then` step in this scenario still names `alias_sentinel`, so the
    // fixture must be seeded even though the scenario fails before reaching it.
    let _ = (world, alias_sentinel);
}
