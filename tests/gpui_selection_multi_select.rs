//! BDD coverage for Shift-click anchor multi-selection without dragging.
//!
//! This binary binds the corresponding scenario in `selection.feature` to the
//! GPUI `GpuiHarness`. It uses `common` for anchor coordinates and modifiers,
//! while `selection_bdd::support` supplies reusable lifecycle state for the
//! ordered clicks and press-time drag observations. Model-only helpers shared
//! by other binaries live in `test_support::selection`.

#[path = "common/gpui_selection_multi_select.rs"]
mod common;
#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "selection_bdd/mutable_scenario_data.rs"]
mod mutable_scenario_data;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "selection_bdd/support.rs"]
mod support;

use common::{
    anchor_to_canvas_point, canvas_bounds, draw_point, read_document, require_draw_shape,
    shift_secondary,
};
use gauss::model::{SelItem, Selection, ShapeId};
use gpui::{Modifiers, MouseButton, TestAppContext, point, px};
use mutable_scenario_data::with_mut_scenario_data;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{
    NoDragPress, ScenarioContext, ScenarioStateCleanup, assert_no_drag_after_press, require_point,
    set_scenario_data, with_scenario_data, with_state, with_visual_cx,
};
use test_support::TestSupportError;

struct ScenarioData {
    shape_id: ShapeId,
    additive_shift_drag_started_after_press: Option<bool>,
    toggle_shift_drag_started_after_press: Option<bool>,
}

#[given("a two-anchor shape is arranged in manipulate mode")]
fn two_anchor_shape_is_arranged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
        let second = point(
            bounds.origin.x + bounds.size.width - px(2.0),
            bounds.origin.y + bounds.size.height - px(2.0),
        );
        draw_point(visual_cx, first);
        draw_point(visual_cx, second);
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "two-anchor selection setup")?;
        let anchor0 =
            shape.path.anchors.first().ok_or_else(|| {
                TestSupportError::missing("anchor 0", "two-anchor selection setup")
            })?;
        let anchor1 =
            shape.path.anchors.get(1).ok_or_else(|| {
                TestSupportError::missing("anchor 1", "two-anchor selection setup")
            })?;
        let anchor0_point = anchor_to_canvas_point(&bounds, anchor0.pos, first);
        let anchor1_point = anchor_to_canvas_point(&bounds, anchor1.pos, second);
        let shape_id = shape.id;
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        with_state(|state| {
            state.points.extend([anchor0_point, anchor1_point]);
        });
        set_scenario_data(ScenarioData {
            shape_id,
            additive_shift_drag_started_after_press: None,
            toggle_shift_drag_started_after_press: None,
        });
        Ok(())
    })
}

fn click_anchor(
    cx: &mut TestAppContext,
    index: usize,
    modifiers: Modifiers,
    record_drag_state: impl FnOnce(&mut ScenarioData, bool),
) -> Result<(), TestSupportError> {
    let point = require_point(index, ScenarioContext::AnchorClick)?;
    with_visual_cx(cx, |visual_cx, view| {
        visual_cx.simulate_mouse_down(point, MouseButton::Left, modifiers);
        visual_cx.run_until_parked();
        let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
        with_mut_scenario_data::<ScenarioData, _>(ScenarioContext::AnchorClick, |data| {
            record_drag_state(data, is_dragging);
        })?;
        visual_cx.simulate_mouse_up(point, MouseButton::Left, modifiers);
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("the first anchor is clicked")]
fn first_anchor_is_clicked(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_anchor(cx, 0, Modifiers::none(), |_, _| {})
}

#[when("the second anchor is shift-clicked")]
fn second_anchor_is_shift_clicked(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_anchor(
        cx,
        1,
        shift_secondary(Modifiers::none()),
        |data, is_dragging| {
            data.additive_shift_drag_started_after_press = Some(is_dragging);
        },
    )
}

#[when("the first anchor is shift-clicked")]
fn first_anchor_is_shift_clicked(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_anchor(
        cx,
        0,
        shift_secondary(Modifiers::none()),
        |data, is_dragging| {
            data.toggle_shift_drag_started_after_press = Some(is_dragging);
        },
    )
}

fn expected_anchor_selection(indices: &[usize]) -> Result<Selection, TestSupportError> {
    let shape_id =
        with_scenario_data::<ScenarioData, _>(ScenarioContext::AnchorSelection, |data| {
            data.shape_id
        })?;
    let mut items = vec![SelItem::Shape(shape_id)];
    items.extend(indices.iter().map(|anchor| SelItem::Anchor {
        shape: shape_id,
        anchor: *anchor,
    }));
    Ok(Selection { items })
}

fn require_selection(
    cx: &mut TestAppContext,
    expected: &Selection,
    context: &str,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).selection().clone());
        if actual != *expected {
            return Err(TestSupportError::expectation(format!(
                "{context}: expected {expected:?}, found {actual:?}"
            )));
        }
        Ok(())
    })
}

#[then("only the first anchor is selected")]
fn only_first_anchor_is_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_selection(
        cx,
        &expected_anchor_selection(&[0])?,
        "first anchor selection",
    )
}

#[then("both anchors are selected")]
fn both_anchors_are_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_selection(cx, &expected_anchor_selection(&[0, 1])?, "multi-selection")
}

#[then("only the second anchor is selected")]
fn only_second_anchor_is_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_selection(cx, &expected_anchor_selection(&[1])?, "toggled selection")
}

#[then("no drag is active")]
fn no_drag_is_active() -> Result<(), TestSupportError> {
    let (additive_shift_drag_started_after_press, toggle_shift_drag_started_after_press) =
        with_scenario_data::<ScenarioData, _>(ScenarioContext::ShiftClickDragState, |data| {
            (
                data.additive_shift_drag_started_after_press,
                data.toggle_shift_drag_started_after_press,
            )
        })?;
    assert_no_drag_after_press(
        additive_shift_drag_started_after_press,
        NoDragPress::AdditiveShiftClick,
    )?;
    assert_no_drag_after_press(
        toggle_shift_drag_started_after_press,
        NoDragPress::ToggleShiftClick,
    )?;
    Ok(())
}

#[scenario(
    path = "tests/features/selection.feature",
    name = "Shift-click toggles multi-selection without dragging",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn shift_click_multi_select(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
