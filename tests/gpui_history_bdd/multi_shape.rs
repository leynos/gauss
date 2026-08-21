//! BDD step bindings for one-entry multi-shape drag history.
//!
//! The steps create two selected shapes, drag them together, and verify that
//! one history entry moves or restores both shapes. The parent integration
//! binary runs the feature scenario with `GpuiHarness`; common document,
//! canvas, history, and durable-shell helpers provide the shared setup.

use std::cell::RefCell;

use crate::common::{
    add_square, assert_shape_translated_by_delta, canvas_bounds, read_document, read_history_len,
    simulate_document_undo,
};
use crate::history_bdd_support::{DurableShell, missing};
use gauss::model::{Document, SelItem, Shape, ShapeId, Vec2};
use gpui::{Modifiers, MouseButton, TestAppContext, px};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::{TestSupportError, TestSupportResult, math};

#[derive(Default)]
struct MultiDragState {
    shell: Option<DurableShell>,
    first: Option<Shape>,
    second: Option<Shape>,
    delta: Option<Vec2>,
    history_before: Option<usize>,
}

thread_local! {
    static STATE: RefCell<MultiDragState> = RefCell::new(MultiDragState::default());
}

/// Apply a closure to the multi-shape drag scenario state.
fn with_state<R>(f: impl FnOnce(&mut MultiDragState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

/// Reset all multi-shape drag state before or after a scenario.
fn reset_state() {
    with_state(|state| *state = MultiDragState::default());
}

struct Cleanup;

impl Drop for Cleanup {
    /// Clear thread-local state when the scenario guard is dropped.
    fn drop(&mut self) {
        reset_state();
    }
}

/// Reset state and return the scenario cleanup guard.
#[fixture]
fn cleanup() -> Cleanup {
    reset_state();
    Cleanup
}

/// Retrieve the durable shell stored by the Given step.
fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| missing("Phase 0 shell"))
}

/// Find a shape by identifier and report a scenario-specific missing error.
fn find_shape<'a>(
    document: &'a Document,
    id: ShapeId,
    context: &str,
) -> TestSupportResult<&'a Shape> {
    document
        .shape(id)
        .ok_or_else(|| TestSupportError::missing("shape", format!("shape {id:?}: {context}")))
}

/// Calculate the centre of a shape's anchor bounding box.
fn shape_bbox_centre(shape: &Shape) -> Result<Vec2, TestSupportError> {
    let first = shape.path.anchors.first().ok_or_else(|| {
        TestSupportError::missing("shape anchor", "computing bounding box centre")
    })?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (first.pos.x, first.pos.y, first.pos.x, first.pos.y);
    for anchor in shape.path.anchors.iter().skip(1) {
        min_x = min_x.min(anchor.pos.x);
        min_y = min_y.min(anchor.pos.y);
        max_x = max_x.max(anchor.pos.x);
        max_y = max_y.max(anchor.pos.y);
    }
    Ok(Vec2::new(
        math::midpoint(min_x, max_x),
        math::midpoint(min_y, max_y),
    ))
}

/// Prepare two selected shapes and record their pre-drag geometry.
#[given("a fresh Phase 0 shell window with two selected shapes")]
fn two_selected_shapes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (first, second, history_before) = shell.with_visual(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let mut document = read_document(visual_cx, view);
        let first_id = add_square(
            &mut document,
            origin.add(Vec2::new(10.0, 10.0)),
            origin.add(Vec2::new(110.0, 110.0)),
        )?;
        let second_id = add_square(
            &mut document,
            origin.add(Vec2::new(160.0, 10.0)),
            origin.add(Vec2::new(260.0, 110.0)),
        )?;
        let first = find_shape(&document, first_id, "before drag")?.clone();
        let second = find_shape(&document, second_id, "before drag")?.clone();
        visual_cx.update(|_window, app| {
            view.update(app, |phase0, view_cx| {
                phase0.enter_manipulate_mode_for_tests();
                phase0.replace_document_for_tests(document);
                phase0.replace_selection_for_tests(gauss::model::Selection {
                    items: vec![SelItem::Shape(first_id), SelItem::Shape(second_id)],
                });
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        Ok((first, second, read_history_len(visual_cx, view)))
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.first = Some(first);
        state.second = Some(second);
        state.delta = Some(Vec2::new(20.0, 10.0));
        state.history_before = Some(history_before);
    });
    Ok(())
}

/// Drag both selected shapes together by the configured delta.
#[when("the selected shapes are dragged together")]
fn drag_selected_shapes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let first = with_state(|state| state.first.clone()).ok_or_else(|| missing("first shape"))?;
    let delta = with_state(|state| state.delta).ok_or_else(|| missing("drag delta"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let viewport = visual_cx.read(|app| view.read(app).viewport());
        let start_model = shape_bbox_centre(&first)?;
        let start_screen = viewport.world_to_screen(start_model);
        let end_screen = viewport.world_to_screen(start_model.add(delta));
        let start = gpui::point(px(start_screen.x), px(start_screen.y));
        let end = gpui::point(px(end_screen.x), px(end_screen.y));
        visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        visual_cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

/// Undo the multi-shape drag document change.
#[when("the last document change is undone")]
fn undo_drag(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

/// Assert that both selected shapes moved by the drag delta.
#[then("both selected shapes move by the drag delta")]
fn shapes_moved(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let delta = with_state(|state| state.delta).ok_or_else(|| missing("drag delta"))?;
    assert_shapes_at_delta(cx, delta)
}

/// Assert that undo restored both selected shapes.
#[then("both selected shapes return to their positions before the drag")]
fn shapes_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_shapes_at_delta(cx, Vec2::ZERO)
}

/// Assert both stored shapes are translated by the supplied delta.
fn assert_shapes_at_delta(cx: &mut TestAppContext, delta: Vec2) -> Result<(), TestSupportError> {
    let first = with_state(|state| state.first.clone()).ok_or_else(|| missing("first shape"))?;
    let second = with_state(|state| state.second.clone()).ok_or_else(|| missing("second shape"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let actual_first = find_shape(&document, first.id, "after drag history action")?;
        let actual_second = find_shape(&document, second.id, "after drag history action")?;
        assert_shape_translated_by_delta(actual_first, &first, delta, "first selected shape")?;
        assert_shape_translated_by_delta(actual_second, &second, delta, "second selected shape")
    })
}

/// Assert the multi-shape drag created exactly one history entry.
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
                "expected exactly one undo entry for multi-shape drag; before={before}, after={actual}"
            )));
        }
        Ok(())
    })
}

/// Run the multi-shape drag undo and restoration feature scenario.
#[scenario(
    path = "tests/features/history_multi_shape_drag_undo.feature",
    name = "Dragging multiple shapes creates one undo entry and undo restores them",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn multi_shape_drag_scenario(#[from(cleanup)] _cleanup: Cleanup) {}
