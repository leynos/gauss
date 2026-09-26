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
//! proves the alias still injects `Ok`; the second, run under
//! `#[should_panic]`, proves a step returning `Err` through the same alias
//! actually fails the scenario. Under the old classification the second test
//! would have failed — the discarded `Err` would leave the scenario green and
//! leave `#[should_panic]` unsatisfied.

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

#[given("an empty document for alias classification")]
fn empty_document_for_alias_classification(world: &mut AliasWorld) {
    world.doc = Document::new();
    world.reached_failure = false;
}

/// A step that succeeds while returning through the `Result` alias.
///
/// The `Ok(())` must be unwrapped and injected as the step's value; the alias
/// must not be treated as an opaque payload.
#[when("a step returns Ok through the result alias")]
fn step_returns_ok_through_result_alias(world: &mut AliasWorld) -> TestSupportResult<()> {
    if !world.doc.visually_eq(&Document::new()) {
        return Err(TestSupportError::expectation(
            "expected a freshly constructed document to match an empty document",
        ));
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

#[then("the alias step is recorded as having succeeded")]
fn alias_step_recorded_as_succeeded(world: &AliasWorld) -> TestSupportResult<()> {
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
fn step_returning_ok_through_result_alias_succeeds(world: AliasWorld) {
    let _ = world;
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
fn step_returning_err_through_result_alias_fails_scenario(world: AliasWorld) {
    let _ = world;
}
