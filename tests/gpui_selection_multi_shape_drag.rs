//! BDD coverage for dragging every shape in a multi-shape selection.
//!
//! This binary binds the corresponding scenario in `selection.feature` to the
//! GPUI `GpuiHarness`. Canvas operations come from `common`, GPUI conversion
//! comes from `selection_coordinates`, durable handles come from
//! `selection_bdd::support`, and model-only shape and selection queries come
//! from `test_support::selection` for reuse across test suites.

mod common;
#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "common/selection_coordinates.rs"]
mod selection_coordinates;
#[path = "selection_bdd/support.rs"]
mod support;

use common::{add_square, assert_shape_translated_by_delta, canvas_bounds, read_document};
use gauss::model::{SelItem, Selection, Shape, ShapeId, Vec2};
use gpui::{Modifiers, MouseButton, TestAppContext};
use rstest_bdd_macros::{given, scenario, then, when};
use selection_coordinates::viewport_to_screen_point;
use serial_test::serial;
use support::{
    ScenarioContext, ScenarioStateCleanup, require_point, set_scenario_data, with_scenario_data,
    with_state, with_visual_cx,
};
use test_support::TestSupportError;
use test_support::selection::{
    require_selection_contains_shapes, require_shape, shape_bbox_centre,
};

struct ScenarioData {
    shape_ids: [ShapeId; 2],
    shapes_before: [Shape; 2],
    delta: Vec2,
}

#[given("two selected squares are arranged")]
fn two_selected_squares_are_arranged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let mut document = read_document(visual_cx, view);
        let first = add_square(
            &mut document,
            origin.add(Vec2::new(10.0, 10.0)),
            origin.add(Vec2::new(110.0, 110.0)),
        )?;
        let second = add_square(
            &mut document,
            origin.add(Vec2::new(160.0, 10.0)),
            origin.add(Vec2::new(260.0, 110.0)),
        )?;
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(Selection {
                    items: vec![SelItem::Shape(first), SelItem::Shape(second)],
                });
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();

        let updated_document = read_document(visual_cx, view);
        let first_shape =
            require_shape(&updated_document, first, "first square before drag")?.clone();
        let second_shape =
            require_shape(&updated_document, second, "second square before drag")?.clone();
        let start_world = shape_bbox_centre(&first_shape)?;
        let delta = Vec2::new(20.0, 10.0);
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
            shape_ids: [first, second],
            shapes_before: [first_shape, second_shape],
            delta,
        });
        Ok(())
    })
}

#[when("the first selected square is pressed")]
fn first_selected_square_is_pressed(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let start = require_point(0, ScenarioContext::SelectedSquarePress)?;
    with_visual_cx(cx, |visual_cx, _view| {
        visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("the first selected square is dragged")]
fn first_selected_square_is_dragged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let end = require_point(1, ScenarioContext::SelectedSquareDragEnd)?;
    with_visual_cx(cx, |visual_cx, _view| {
        visual_cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("both squares remain selected")]
fn both_squares_remain_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let ids = with_scenario_data::<ScenarioData, _>(ScenarioContext::SelectedSquares, |data| {
        data.shape_ids
    })?;
    with_visual_cx(cx, |visual_cx, view| {
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        require_selection_contains_shapes(&selection, &ids, "multi-shape drag")
    })
}

#[then("both squares move by the drag delta")]
fn both_squares_move_by_delta(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let snapshot =
        with_scenario_data::<ScenarioData, _>(ScenarioContext::MultiShapeDragSnapshot, |data| {
            (data.shape_ids, data.shapes_before.clone(), data.delta)
        })?;
    let ([first_id, second_id], [first_before, second_before], delta) = snapshot;
    with_visual_cx(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let first = require_shape(&document, first_id, "first square after drag")?;
        let second = require_shape(&document, second_id, "second square after drag")?;
        assert_shape_translated_by_delta(first, &first_before, delta, "first selected square")?;
        assert_shape_translated_by_delta(second, &second_before, delta, "second selected square")
    })
}

#[then("no drag is active")]
fn no_drag_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        if visual_cx.read(|app| view.read(app).is_dragging()) {
            return Err(TestSupportError::expectation(
                "multi-shape drag gesture remained active".to_owned(),
            ));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/selection.feature",
    name = "Dragging one selected shape moves the full selection",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn multi_shape_drag(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
