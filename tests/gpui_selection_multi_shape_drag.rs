//! Behavioural coverage for dragging a multi-shape selection.

mod common;
#[path = "selection_bdd/support.rs"]
mod support;

use common::{add_square, assert_shape_translated_by_delta, canvas_bounds, read_document};
use gauss::model::{SelItem, Selection, Vec2};
use gpui::{Modifiers, MouseButton, TestAppContext};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{
    ScenarioStateCleanup, require_point, require_selection_contains_shapes, require_shape,
    require_shape_id, shape_bbox_centre, viewport_to_screen_point, with_state, with_visual_cx,
};
use test_support::TestSupportError;

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
            state.shape_ids.extend([first, second]);
            state.shapes_before.extend([first_shape, second_shape]);
            state
                .points
                .push(viewport_to_screen_point(viewport, start_world));
            state
                .points
                .push(viewport_to_screen_point(viewport, start_world.add(delta)));
            state.delta = Some(delta);
        });
        Ok(())
    })
}

#[when("the first selected square is pressed")]
fn first_selected_square_is_pressed(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let start = require_point(0, "selected square press")?;
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
    let end = require_point(1, "selected square drag end")?;
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
    let ids = [
        require_shape_id(0, "first selected square")?,
        require_shape_id(1, "second selected square")?,
    ];
    with_visual_cx(cx, |visual_cx, view| {
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        require_selection_contains_shapes(&selection, &ids, "multi-shape drag")
    })
}

#[then("both squares move by the drag delta")]
fn both_squares_move_by_delta(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let snapshot = with_state(|state| {
        (
            state.shape_ids.clone(),
            state.shapes_before.clone(),
            state.delta,
        )
    });
    let ([first_id, second_id], [first_before, second_before], Some(delta)) =
        (snapshot.0.as_slice(), snapshot.1.as_slice(), snapshot.2)
    else {
        return Err(TestSupportError::missing(
            "multi-shape drag snapshot",
            "recorded by the arrangement step",
        ));
    };
    with_visual_cx(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let first = require_shape(&document, *first_id, "first square after drag")?;
        let second = require_shape(&document, *second_id, "second square after drag")?;
        assert_shape_translated_by_delta(first, first_before, delta, "first selected square")?;
        assert_shape_translated_by_delta(second, second_before, delta, "second selected square")
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
