//! BDD coverage for selection-only undo and redo history.

#[path = "common/gpui_history_selection_history.rs"]
mod common;
#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open.rs"]
mod history_bdd_support_open;
#[path = "common/scenario_state.rs"]
mod scenario_state;

use common::{
    anchor_to_canvas_point, canvas_bounds, click_left_and_wait, draw_point, require_draw_shape,
    simulate_escape,
};
use gauss::model::Selection;
use gpui::{Pixels, Point, TestAppContext, point, px};
use history_bdd_support::{DurableShell, missing};
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

/// Dispatch the selection-undo action and wait for the app to settle.
fn simulate_selection_undo(visual_cx: &mut gpui::VisualTestContext) {
    visual_cx.dispatch_action(gauss::ui::GpuiSelectionUndo);
    visual_cx.run_until_parked();
}

/// Dispatch the selection-redo action and wait for the app to settle.
fn simulate_selection_redo(visual_cx: &mut gpui::VisualTestContext) {
    visual_cx.dispatch_action(gauss::ui::GpuiSelectionRedo);
    visual_cx.run_until_parked();
}

#[derive(Default, ScenarioState)]
struct SelectionState {
    shell: Slot<DurableShell>,
    select_point: Slot<Point<Pixels>>,
    clear_point: Slot<Point<Pixels>>,
    selected: Slot<Selection>,
    cleared: Slot<Selection>,
    shape_count: Slot<usize>,
}

crate::scenario_state!(SelectionState);

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.get()).ok_or_else(|| missing("Phase 0 shell"))
}

#[given("a fresh Phase 0 shell window with a drawn path in manipulate mode")]
#[expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
fn drawn_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (select_point, clear_point, shape_count) = shell.with_visual(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
        let second = point(
            first.x + px((width - 4.0).clamp(1.0, 40.0)),
            first.y + px((height - 4.0).clamp(1.0, 24.0)),
        );
        draw_point(visual_cx, first);
        draw_point(visual_cx, second);
        let document = visual_cx.read(|app| view.read(app).document().clone());
        let shape = require_draw_shape(&document, "after drawing")?;
        let anchor = shape
            .path
            .anchors
            .first()
            .map_or(gauss::model::Vec2::ZERO, |item| item.pos);
        let select_point = anchor_to_canvas_point(&bounds, anchor, first);
        let clear_point = point(
            bounds.origin.x + px((width - 12.0).max(1.0)),
            bounds.origin.y + px((height - 12.0).max(1.0)),
        );
        simulate_escape(visual_cx);
        Ok((select_point, clear_point, document.len()))
    })?;
    with_state(|state| {
        state.shell.set(shell);
        state.select_point.set(select_point);
        state.clear_point.set(clear_point);
        state.shape_count.set(shape_count);
    });
    Ok(())
}

#[when("the first anchor is selected")]
fn select_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point =
        with_state(|state| state.select_point.get()).ok_or_else(|| missing("select point"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        click_left_and_wait(visual_cx, point);
        let selected = visual_cx.read(|app| view.read(app).selection().clone());
        with_state(|state| state.selected.set(selected));
        Ok(())
    })
}

#[when("the selection is cleared")]
fn clear_selection(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point =
        with_state(|state| state.clear_point.get()).ok_or_else(|| missing("clear point"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        click_left_and_wait(visual_cx, point);
        let cleared = visual_cx.read(|app| view.read(app).selection().clone());
        with_state(|state| state.cleared.set(cleared));
        Ok(())
    })
}

#[when("the last selection change is undone")]
fn undo_selection(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_selection_undo(visual_cx);
        Ok(())
    })
}

#[when("the last selection change is redone")]
fn redo_selection(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_selection_redo(visual_cx);
        Ok(())
    })
}

#[then("the selection is not empty")]
fn selection_not_empty(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_selection(cx, false, None)
}

#[then("the selection is empty")]
fn selection_empty(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| state.cleared.get());
    assert_selection(cx, true, expected.as_ref())
}

#[then("the previous selection is restored")]
fn selection_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected =
        with_state(|state| state.selected.get()).ok_or_else(|| missing("selected snapshot"))?;
    assert_selection(cx, false, Some(&expected))
}

fn assert_selection(
    cx: &mut TestAppContext,
    should_be_empty: bool,
    expected: Option<&Selection>,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).selection().clone());
        if actual.is_empty() != should_be_empty || expected.is_some_and(|value| value != &actual) {
            return Err(TestSupportError::expectation(format!(
                "unexpected selection state: {actual:?}"
            )));
        }
        Ok(())
    })
}

#[then("the document shape count is unchanged")]
fn document_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected =
        with_state(|state| state.shape_count.get()).ok_or_else(|| missing("shape count"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).document().len());
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "selection undo/redo changed document shape count from {expected} to {actual}"
            )));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/history_selection_history.feature",
    name = "Selection undo and redo do not change the document",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn selection_history_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
