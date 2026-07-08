//! GPUI behavioural test for Phase 0 draw-mode undo/redo, expressed as a
//! Gherkin scenario driven through the first-party
//! [`rstest_bdd_harness_gpui::GpuiHarness`].
//!
//! This is the migration proof-point for `rstest-bdd 0.6.0-beta3`: it replaces
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
//! API note: this crate consumes the *published* `gpui 0.2.2`, so
//! `VisualTestContext::from_window` returns a `VisualTestContext` by value and
//! `window_handle()` is a `gpui::VisualContext` trait method. These differ from
//! the vendored-fork shapes in the upstream user's guide.

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
/// Published `gpui 0.2.2`: `from_window` takes `&TestAppContext` and returns a
/// `VisualTestContext` by value.
fn with_visual_cx<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> R,
) -> R {
    let handles = with_state(|state| (state.entity.clone(), state.window));
    let (Some(entity), Some(window)) = handles else {
        panic!("durable view and window handles should be set by the given step");
    };
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    f(&mut visual_cx, &entity)
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    reset_state_before_assignment();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    let Ok((first, second)) = canvas_points(visual_cx) else {
        panic!("canvas points should be available");
    };
    with_state(|state| {
        state.entity = Some(entity);
        state.window = Some(window);
        state.first = Some(first);
        state.second = Some(second);
    });
}

#[when("the first anchor is placed")]
fn place_first_anchor(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    let Some(first) = with_state(|state| state.first) else {
        panic!("first canvas point should be set");
    };
    with_visual_cx(cx, |visual_cx, _view| {
        click_canvas_and_wait(visual_cx, first);
    });
}

#[when("the second anchor is placed")]
fn place_second_anchor(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    let Some(second) = with_state(|state| state.second) else {
        panic!("second canvas point should be set");
    };
    with_visual_cx(cx, |visual_cx, _view| {
        click_canvas_and_wait(visual_cx, second);
    });
}

#[when("the last change is undone")]
fn undo_last_change(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    with_visual_cx(cx, |visual_cx, _view| simulate_document_undo(visual_cx));
}

#[when("the last change is redone")]
fn redo_last_change(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    with_visual_cx(cx, |visual_cx, _view| simulate_document_redo(visual_cx));
}

#[then("the draw shape anchor count is {count:usize}")]
fn draw_shape_anchor_count(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: usize,
) {
    with_visual_cx(cx, |visual_cx, view| {
        let doc = read_document(visual_cx, view);
        let Ok(shape) = require_draw_shape(&doc, "draw_undo scenario") else {
            panic!("draw shape should be present");
        };
        assert_eq!(
            shape.path.anchors.len(),
            count,
            "unexpected anchor count on the draw shape"
        );
    });
}

#[then("the draw shape is absent")]
fn draw_shape_absent(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    with_visual_cx(cx, |visual_cx, view| {
        let doc = read_document(visual_cx, view);
        assert!(
            find_draw_shape(&doc).is_none(),
            "draw shape should be absent after undoing the first click"
        );
    });
}

#[scenario(
    path = "tests/features/draw_undo.feature",
    name = "Draw clicks add anchors and undo removes them",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn draw_undo_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
