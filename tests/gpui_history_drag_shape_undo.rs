//! BDD coverage for shape drag history.

#[path = "common/gpui_history_drag_shape_undo.rs"]
mod common;
#[path = "gpui_history_bdd/support.rs"]
mod history_support;

use std::cell::RefCell;

use common::{
    assert_shape_translated_by_delta, canvas_drag_scenario, draw_point, read_document,
    read_history_len, require_draw_shape, simulate_document_undo,
    switch_to_manipulate_mode_and_verify,
};
use gauss::model::{SelItem, Shape, Vec2};
use gpui::{Modifiers, MouseButton, Pixels, Point, TestAppContext, point, px};
use history_support::{DurableShell, missing};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::{TestSupportError, math};

#[derive(Default)]
struct DragState {
    shell: Option<DurableShell>,
    original: Option<Shape>,
    delta: Option<Vec2>,
    start: Option<Point<Pixels>>,
    end: Option<Point<Pixels>>,
    history_before: Option<usize>,
}

thread_local! {
    static STATE: RefCell<DragState> = RefCell::new(DragState::default());
}

fn with_state<R>(f: impl FnOnce(&mut DragState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = DragState::default());
}

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        reset_state();
    }
}

#[fixture]
fn cleanup() -> Cleanup {
    reset_state();
    Cleanup
}

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| missing("Phase 0 shell"))
}

#[given("a fresh Phase 0 shell window with a drawn shape in manipulate mode")]
fn drawn_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let values = shell.with_visual(cx, |visual_cx, view| {
        let drag = canvas_drag_scenario(visual_cx, 20.0, 10.0)?;
        draw_point(visual_cx, drag.first);
        draw_point(visual_cx, drag.second);
        let document = read_document(visual_cx, view);
        let original = require_draw_shape(&document, "after drawing two points")?.clone();
        switch_to_manipulate_mode_and_verify(visual_cx, view, drag.first);
        let start = point(
            px(math::midpoint(
                f32::from(drag.first.x),
                f32::from(drag.second.x),
            )),
            px(math::midpoint(
                f32::from(drag.first.y),
                f32::from(drag.second.y),
            )),
        );
        let end = point(start.x + px(drag.delta.x), start.y + px(drag.delta.y));
        Ok((
            original,
            drag.delta,
            start,
            end,
            read_history_len(visual_cx, view),
        ))
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.original = Some(values.0);
        state.delta = Some(values.1);
        state.start = Some(values.2);
        state.end = Some(values.3);
        state.history_before = Some(values.4);
    });
    Ok(())
}

#[when("the drawn shape is dragged")]
fn drag_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let start = with_state(|state| state.start).ok_or_else(|| missing("drag start"))?;
    let end = with_state(|state| state.end).ok_or_else(|| missing("drag end"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        let original = with_state(|state| state.original.clone()).ok_or_else(|| missing("original shape"))?;
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        let is_selected = selection.items.iter().any(|item| match item {
            SelItem::Shape(id) => *id == original.id,
            SelItem::Segment { shape, .. } => *shape == original.id,
            _ => false,
        });
        if !is_selected || !visual_cx.read(|app| view.read(app).is_dragging()) {
            return Err(TestSupportError::expectation(format!(
                "expected mouse down to select the shape and start dragging; selection={selection:?}"
            )));
        }
        visual_cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        if visual_cx.read(|app| view.read(app).is_dragging()) {
            return Err(TestSupportError::expectation(
                "expected mouse up to end the active drag gesture",
            ));
        }
        Ok(())
    })
}

#[when("the last document change is undone")]
fn undo_drag(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[then("the drawn shape moves by the drag delta")]
fn shape_moved(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_translation(
        cx,
        with_state(|state| state.delta).ok_or_else(|| missing("drag delta"))?,
    )
}

#[then("the drawn shape returns to its position before the drag")]
fn shape_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_translation(cx, Vec2::ZERO)
}

fn assert_translation(cx: &mut TestAppContext, delta: Vec2) -> Result<(), TestSupportError> {
    let original =
        with_state(|state| state.original.clone()).ok_or_else(|| missing("original shape"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after shape drag history action")?;
        assert_shape_translated_by_delta(shape, &original, delta, "shape drag scenario")
    })
}

#[then("the drawn shape remains selected")]
fn shape_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let id = with_state(|state| state.original.as_ref().map(|shape| shape.id))
        .ok_or_else(|| missing("original shape"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        if !selection.items.iter().any(|item| match item {
            SelItem::Shape(shape) | SelItem::Segment { shape, .. } => *shape == id,
            _ => false,
        }) {
            return Err(TestSupportError::expectation(format!(
                "expected dragged shape to remain selected; selection={selection:?}"
            )));
        }
        Ok(())
    })
}

#[then("the document history has gained 1 entry")]
fn history_gained_one(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let before =
        with_state(|state| state.history_before).ok_or_else(|| missing("history length"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = read_history_len(visual_cx, view);
        if actual != before + 1 {
            return Err(TestSupportError::expectation(format!(
                "expected exactly one undo entry for shape drag; before={before}, after={actual}"
            )));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/history_drag_shape_undo.feature",
    name = "Dragging a shape creates one undo entry and undo restores it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn drag_shape_scenario(#[from(cleanup)] _cleanup: Cleanup) {}
