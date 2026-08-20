//! BDD coverage for dragging an unselected shape by its bounding box.
//!
//! This binary binds the corresponding scenario in `selection.feature` to the
//! GPUI `GpuiHarness`. It uses `common` for canvas interactions,
//! `selection_coordinates` for GPUI conversion, shared lifecycle state from
//! `selection_bdd::support`, and reusable model queries from
//! `test_support::selection` to preserve the press-time selection rule.

#[path = "common/gpui_selection_bbox_drag_requires_selection.rs"]
mod common;
#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "selection_bdd/mutable_scenario_data.rs"]
mod mutable_scenario_data;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "common/selection_coordinates.rs"]
mod selection_coordinates;
#[path = "selection_bdd/support.rs"]
mod support;

use common::{add_square, assert_shape_translated_by_delta, canvas_bounds, read_document};
use gauss::model::{Document, SelItem, Selection, Shape, ShapeId, Vec2};
use gpui::{Modifiers, MouseButton, TestAppContext};
use mutable_scenario_data::with_mut_scenario_data;
use rstest_bdd_macros::{given, scenario, then, when};
use selection_coordinates::viewport_to_screen_point;
use serial_test::serial;
use support::{
    NoDragPress, ScenarioContext, ScenarioStateCleanup, assert_no_drag_after_press, require_point,
    set_scenario_data, with_scenario_data, with_state, with_visual_cx,
};
use test_support::TestSupportError;
use test_support::selection::{require_shape, shape_bbox_centre};

struct ScenarioData {
    shape_id: ShapeId,
    shape_before: Shape,
    drag_started_after_press: Option<bool>,
    selection_after_press: Option<Selection>,
}

#[given("an unselected square is arranged")]
fn unselected_square_is_arranged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let mut document = Document::new();
        let shape_id = add_square(
            &mut document,
            origin.add(Vec2::new(10.0, 10.0)),
            origin.add(Vec2::new(110.0, 110.0)),
        )?;
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(Selection::empty());
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();

        let updated_document = read_document(visual_cx, view);
        let shape = require_shape(&updated_document, shape_id, "before unselected drag")?.clone();
        let start_world = shape_bbox_centre(&shape)?;
        let delta = Vec2::new(25.0, 15.0);
        let viewport = visual_cx.read(|app| view.read(app).viewport());
        with_state(|state| {
            state
                .points
                .push(viewport_to_screen_point(viewport, start_world));
            state
                .points
                .push(viewport_to_screen_point(viewport, start_world.add(delta)));
        });
        set_scenario_data(ScenarioData {
            shape_id,
            shape_before: shape,
            drag_started_after_press: None,
            selection_after_press: None,
        });
        Ok(())
    })
}

#[when("the unselected square is dragged by its bounding box")]
fn unselected_square_is_dragged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let start = require_point(0, ScenarioContext::UnselectedDragStart)?;
    let end = require_point(1, ScenarioContext::UnselectedDragEnd)?;
    with_visual_cx(cx, |visual_cx, view| {
        visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        with_mut_scenario_data::<ScenarioData, _>(ScenarioContext::UnselectedDrag, |data| {
            data.drag_started_after_press = Some(is_dragging);
            data.selection_after_press = Some(selection);
        })?;
        visual_cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the square is selected")]
fn square_is_selected() -> Result<(), TestSupportError> {
    let (shape_id, selection_after_press) =
        with_scenario_data::<ScenarioData, _>(ScenarioContext::SelectedSquare, |data| {
            (data.shape_id, data.selection_after_press.clone())
        })?;
    let selection = selection_after_press.ok_or_else(|| {
        TestSupportError::missing("selection after press", "recorded by the drag step")
    })?;
    if !selection.contains(&SelItem::Shape(shape_id)) {
        return Err(TestSupportError::expectation(format!(
            "expected square to be selected; selection={selection:?}"
        )));
    }
    Ok(())
}

#[then("no drag starts before the square is preselected")]
fn no_drag_starts_before_preselection() -> Result<(), TestSupportError> {
    let drag_started_after_press =
        with_scenario_data::<ScenarioData, _>(ScenarioContext::DragStateAfterPress, |data| {
            data.drag_started_after_press
        })?;
    assert_no_drag_after_press(drag_started_after_press, NoDragPress::UnselectedBoundingBox)
}

#[then("the square remains unchanged")]
fn square_remains_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (shape_id, original) =
        with_scenario_data::<ScenarioData, _>(ScenarioContext::UnchangedSquare, |data| {
            (data.shape_id, data.shape_before.clone())
        })?;
    with_visual_cx(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let current = require_shape(&document, shape_id, "after unselected drag")?;
        assert_shape_translated_by_delta(
            current,
            &original,
            Vec2::ZERO,
            "unselected bounding-box drag",
        )
    })
}

#[scenario(
    path = "tests/features/selection.feature",
    name = "Dragging an unselected shape selects without moving it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn bbox_drag_requires_preselection(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
