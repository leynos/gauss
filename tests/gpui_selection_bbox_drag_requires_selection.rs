//! Behavioural coverage for bounding-box drags of unselected shapes.

mod common;
#[path = "selection_bdd/support.rs"]
mod support;

use common::{add_square, assert_shape_translated_by_delta, canvas_bounds, read_document};
use gauss::model::{Document, SelItem, Selection, Vec2};
use gpui::{Modifiers, MouseButton, TestAppContext};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{
    ScenarioStateCleanup, require_point, require_shape, require_shape_id, shape_bbox_centre,
    viewport_to_screen_point, with_state, with_visual_cx,
};
use test_support::TestSupportError;

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
            state.shape_ids.push(shape_id);
            state.shapes_before.push(shape);
            state
                .points
                .push(viewport_to_screen_point(viewport, start_world));
            state
                .points
                .push(viewport_to_screen_point(viewport, start_world.add(delta)));
        });
        Ok(())
    })
}

#[when("the unselected square is dragged by its bounding box")]
fn unselected_square_is_dragged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let start = require_point(0, "unselected drag start")?;
    let end = require_point(1, "unselected drag end")?;
    with_visual_cx(cx, |visual_cx, view| {
        visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
        with_state(|state| state.drag_started_after_press = Some(is_dragging));
        visual_cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the square is selected")]
fn square_is_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shape_id = require_shape_id(0, "selected square")?;
    with_visual_cx(cx, |visual_cx, view| {
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        if !selection.contains(&SelItem::Shape(shape_id)) {
            return Err(TestSupportError::expectation(format!(
                "expected square to be selected; selection={selection:?}"
            )));
        }
        Ok(())
    })
}

#[then("no drag starts before the square is preselected")]
fn no_drag_starts_before_preselection() -> Result<(), TestSupportError> {
    match with_state(|state| state.drag_started_after_press) {
        Some(false) => Ok(()),
        Some(true) => Err(TestSupportError::expectation(
            "unselected bounding-box press started a drag".to_owned(),
        )),
        None => Err(TestSupportError::missing(
            "drag state after press",
            "recorded by the drag step",
        )),
    }
}

#[then("the square remains unchanged")]
fn square_remains_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shape_id = require_shape_id(0, "unchanged square")?;
    let original = with_state(|state| state.shapes_before.first().cloned()).ok_or_else(|| {
        TestSupportError::missing("original square", "recorded by the arrangement step")
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
