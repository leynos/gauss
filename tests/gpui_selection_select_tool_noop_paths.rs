//! BDD coverage for manipulate-mode pointer paths that must be no-ops.
//!
//! This binary binds the right-click and zero-delta scenarios in
//! `selection.feature` to the GPUI `GpuiHarness`. It reuses pointer and history
//! utilities from `common` and durable lifecycle state from
//! `selection_bdd::support` to compare state before and after. Model-only
//! helpers shared by other binaries live in `test_support::selection`.

mod common;
#[path = "selection_bdd/support.rs"]
pub mod support;

use common::{
    canvas_bounds, canvas_drag_scenario, draw_point, read_history_len, read_selection,
    switch_to_manipulate_mode_and_verify,
};
use gpui::{Modifiers, MouseButton, TestAppContext, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, require_point, with_state, with_visual_cx};
use test_support::TestSupportError;

#[derive(Default)]
struct ScenarioData {
    selection_before: Option<gauss::model::Selection>,
    history_before: Option<usize>,
}

#[given("manipulate mode is active")]
fn manipulate_mode_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let click_point = point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0));
        switch_to_manipulate_mode_and_verify(visual_cx, view, click_point)?;
        let selection = read_selection(visual_cx, view);
        with_state(|state| state.points.push(click_point));
        support::set_scenario_data(ScenarioData {
            selection_before: Some(selection),
            ..ScenarioData::default()
        });
        Ok(())
    })
}

#[when("the canvas is right-clicked")]
fn canvas_is_right_clicked(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point = require_point(0, "right-click point")?;
    with_visual_cx(cx, |visual_cx, _view| {
        visual_cx.simulate_mouse_down(point, MouseButton::Right, Modifiers::none());
        visual_cx.simulate_mouse_up(point, MouseButton::Right, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[given("a drawn shape is selected in manipulate mode")]
fn drawn_shape_is_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let drag = canvas_drag_scenario(visual_cx, 18.0, 12.0)?;
        draw_point(visual_cx, drag.first);
        draw_point(visual_cx, drag.second);
        switch_to_manipulate_mode_and_verify(visual_cx, view, drag.first)?;
        let history_before = read_history_len(visual_cx, view);
        let selection_before = read_selection(visual_cx, view);
        with_state(|state| state.points.push(drag.first));
        support::set_scenario_data(ScenarioData {
            selection_before: Some(selection_before),
            history_before: Some(history_before),
        });
        Ok(())
    })
}

#[when("the selected point is dragged by zero distance")]
fn selected_point_is_dragged_by_zero_distance(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point = require_point(0, "zero-delta drag point")?;
    with_visual_cx(cx, |visual_cx, _view| {
        visual_cx.simulate_mouse_down(point, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_move(point, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(point, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the selection is unchanged")]
fn selection_is_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = support::with_scenario_data::<ScenarioData, _>("selection snapshot", |data| {
        data.selection_before.clone()
    })?
    .ok_or_else(|| {
        TestSupportError::missing("selection snapshot", "recorded by the arrangement step")
    })?;
    with_visual_cx(cx, |visual_cx, view| {
        let actual = read_selection(visual_cx, view);
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "expected unchanged selection {expected:?}; found {actual:?}"
            )));
        }
        Ok(())
    })
}

#[then("the document history length is unchanged")]
fn document_history_length_is_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = support::with_scenario_data::<ScenarioData, _>("history length", |data| {
        data.history_before
    })?
    .ok_or_else(|| {
        TestSupportError::missing("history length", "recorded by the arrangement step")
    })?;
    with_visual_cx(cx, |visual_cx, view| {
        let actual = read_history_len(visual_cx, view);
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "expected history length {expected}; found {actual}"
            )));
        }
        Ok(())
    })
}

#[then("no drag is active")]
fn no_drag_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        if visual_cx.read(|app| view.read(app).is_dragging()) {
            return Err(TestSupportError::expectation(
                "expected pointer gesture to leave drag state idle".to_owned(),
            ));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/selection.feature",
    name = "Right-clicking in manipulate mode is a no-op",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn right_click_is_noop(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/selection.feature",
    name = "A zero-delta drag is a no-op",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn zero_delta_drag_is_noop(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
