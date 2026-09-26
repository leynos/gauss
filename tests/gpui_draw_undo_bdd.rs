//! GPUI behavioural test for Phase 0 draw-mode undo/redo, expressed as a
//! Gherkin scenario driven through the first-party
//! [`rstest_bdd_harness_gpui::GpuiHarness`].
//!
//! This is the migration proof-point for `rstest-bdd 0.6.0`: it replaces
//! the raw `#[gpui::test]` `draw_click_adds_points_and_undo_removes` function
//! that previously lived in `tests/gpui_history_draw_undo.rs`.
//!
//! The scenario shares durable handles — a [`gpui::Entity<Phase0Shell>`] view
//! and the [`gpui::AnyWindowHandle`] that owns it — across steps, and each step
//! also needs mutable access to the harness-provided [`gpui::TestAppContext`].
//! The v0.6 `StepContext` cannot lend two mutable borrows in one step, so the
//! durable handles live in a `thread_local!` cell (the interim stateful-GPUI
//! playbook) rather than a second fixture. Each such scenario is `#[serial]`
//! and observes a two-sided reset protocol: a reset before the first assignment
//! of fresh handles, and a `Drop`-based reset after the scenario ends.
//!
//! Steps and their helpers are fallible: they propagate failures with `?` or an
//! explicit `Err`, rather than panicking. An `Err` returned from a step aborts
//! the scenario and fails the test, so the `Then` steps are genuine assertions.
//!
//! Step signatures spell out `Result<(), TestSupportError>` rather than the
//! `TestSupportResult` type alias. That was originally a workaround for a
//! `0.6.0-beta3` defect: the step macro classified the return type
//! syntactically and did *not* see through a `Result` type alias, so with the
//! alias it treated the returned value as an opaque payload and silently
//! discarded the `Err`, producing a false green. `rstest-bdd` 0.6.0 resolves
//! aliases by their concrete type at runtime, so the workaround is no longer
//! needed and the alias would now be correct — and is pinned by
//! `crates/gauss-core/tests/result_alias_bdd.rs`. The spelled-out form is kept
//! here only to avoid churning signatures that no longer need a change.
//!
//! The scenario itself returns `()`, not a `Result`. A unit scenario still
//! propagates step `Err`s (validated). A fallible scenario return also works
//! under 0.6.0 — the generated GPUI boundary consumes it and panics on `Err` —
//! but the macro rewrites the signature to unit and the resulting `Ok(())`
//! needs an explicit `Ok::<(), TestSupportError>(())` turbofish for the
//! dropped return type to infer, so the unit form stays simpler.
//!
//! API note: this crate consumes the *published* `gpui 0.2.2`, so
//! `VisualTestContext::from_window` returns a `VisualTestContext` by value and
//! `window_handle()` is a `gpui::VisualContext` trait method. These differ from
//! the vendored-fork shapes in the upstream user's guide.

#[path = "common/gpui_draw_undo_bdd.rs"]
mod common;

use std::cell::RefCell;

use common::{
    canvas_points, click_canvas_and_wait, ensure_initial_draw, find_draw_shape, init_test_app,
    read_document, require_draw_shape, simulate_document_redo, simulate_document_undo,
};
use gauss::ui::Phase0Shell;
use gpui::{
    AnyWindowHandle, Entity, Pixels, Point, TestAppContext, VisualContext, VisualTestContext,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::{TestSupportError, TestSupportResult};

/// Durable, per-scenario handles shared across steps.
///
/// `VisualTestContext` deliberately is *not* stored here: it borrows from the
/// `TestAppContext` it was created against, and each step is handed a fresh
/// `&mut TestAppContext` by the harness. Only the cheap, durable `Entity` and
/// window handle survive between steps; the visual context is rebuilt per step.
#[derive(Default)]
struct ScenarioState {
    entity: Option<Entity<Phase0Shell>>,
    window: Option<AnyWindowHandle>,
    first: Option<Point<Pixels>>,
    second: Option<Point<Pixels>>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

fn with_state<R>(f: impl FnOnce(&mut ScenarioState) -> R) -> R {
    SCENARIO_STATE.with(|cell| f(&mut cell.borrow_mut()))
}

fn reset_state_after_scenario() {
    SCENARIO_STATE.with(|cell| *cell.borrow_mut() = ScenarioState::default());
}

fn reset_state_before_assignment() {
    // Reset before assigning the next scenario's handles so a reused serial
    // test thread cannot observe handles left by a failed or skipped scenario.
    reset_state_after_scenario();
}

/// `Drop` guard that clears the thread-local state on every scenario exit path
/// (success, assertion failure, and panic alike).
struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state_after_scenario();
    }
}

#[fixture]
fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

/// Rebuild a fresh `VisualTestContext` from the stored window handle and the
/// harness-provided `TestAppContext`, then run `f` with the durable view entity.
///
/// Returns `Err` when the durable handles have not been assigned (a scenario
/// ordering invariant), so callers can propagate with `?` rather than panic.
///
/// Published `gpui 0.2.2`: `from_window` takes `&TestAppContext` and returns a
/// `VisualTestContext` by value.
fn with_visual_cx<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
) -> TestSupportResult<R> {
    let handles = with_state(|state| (state.entity.clone(), state.window));
    let (Some(entity), Some(window)) = handles else {
        return Err(TestSupportError::missing(
            "scenario handles",
            "durable view and window handles set by the given step",
        ));
    };
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    f(&mut visual_cx, &entity)
}

#[given("a fresh Phase 0 shell window")]
// Spelled out as `Result<..>` rather than the `TestSupportResult` alias; the
// same applies to every step below. The alias is also correct under 0.6.0 (see
// the module docs), so the explicit form is retained only to avoid churning
// signatures that no longer need a change.
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state_before_assignment();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    let (first, second) = canvas_points(visual_cx)?;
    with_state(|state| {
        state.entity = Some(entity);
        state.window = Some(window);
        state.first = Some(first);
        state.second = Some(second);
    });
    Ok(())
}

#[when("the first anchor is placed")]
fn place_first_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let Some(first) = with_state(|state| state.first) else {
        return Err(TestSupportError::missing(
            "first canvas point",
            "set by the given step",
        ));
    };
    with_visual_cx(cx, |visual_cx, _view| {
        click_canvas_and_wait(visual_cx, first);
        Ok(())
    })
}

#[when("the second anchor is placed")]
fn place_second_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let Some(second) = with_state(|state| state.second) else {
        return Err(TestSupportError::missing(
            "second canvas point",
            "set by the given step",
        ));
    };
    with_visual_cx(cx, |visual_cx, _view| {
        click_canvas_and_wait(visual_cx, second);
        Ok(())
    })
}

#[when("the last change is undone")]
fn undo_last_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[when("the last change is redone")]
fn redo_last_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, _view| {
        simulate_document_redo(visual_cx);
        Ok(())
    })
}

#[then("the draw shape anchor count is {count:usize}")]
fn draw_shape_anchor_count(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: usize,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let doc = read_document(visual_cx, view);
        let shape = require_draw_shape(&doc, "draw_undo scenario")?;
        let actual = shape.path.anchors.len();
        if actual != count {
            return Err(TestSupportError::expectation(format!(
                "expected the draw shape to have {count} anchor(s), found {actual}"
            )));
        }
        Ok(())
    })
}

#[then("the draw shape is absent")]
fn draw_shape_absent(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let doc = read_document(visual_cx, view);
        if find_draw_shape(&doc).is_some() {
            return Err(TestSupportError::expectation(
                "expected the draw shape to be absent after undoing the first click".to_owned(),
            ));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/draw_undo.feature",
    name = "Draw clicks add anchors and undo removes them",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
// Kept unit-returning: a fallible scenario return also works under 0.6.0 (the
// generated GPUI boundary consumes it and panics on `Err`), but the macro
// erases the declared return type, so the trailing `Ok(())` would need an
// explicit `Ok::<(), TestSupportError>(())` turbofish. See the module docs.
fn draw_undo_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
