//! BDD coverage for selecting a shape through its bounding box.
//!
//! This binary binds the corresponding press-time scenario in
//! `selection.feature` to the GPUI `GpuiHarness`. It uses `common` for canvas
//! geometry and `selection_bdd::support` for reusable lifecycle state, then
//! records the shell selection before mouse-up. Model-only helpers shared by
//! other binaries live in `test_support::selection`.

mod common;
#[path = "selection_bdd/support.rs"]
pub mod support;

use common::{canvas_bounds, read_selection};
use gauss::model::{Document, SelItem, Selection, ShapeId, Vec2};
use gauss_core::test_helpers::square_shape;
use gpui::{Modifiers, MouseButton, TestAppContext, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, require_point, with_state, with_visual_cx};
use test_support::{TestSupportError, math};

struct ScenarioData {
    shape_id: ShapeId,
    selection_after_press: Option<Selection>,
}

#[given("an unselected square is arranged for bounding-box selection")]
fn square_is_arranged_for_bbox_selection(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let min = origin.add(Vec2::new(20.0, 20.0));
        let max = origin.add(Vec2::new(160.0, 160.0));
        let centre = Vec2::new(math::midpoint(min.x, max.x), math::midpoint(min.y, max.y));
        let mut document = Document::new();
        let shape_id = document.append_shape(square_shape(ShapeId::default(), min, max));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(Selection::empty());
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        let selection = read_selection(visual_cx, view);
        if !selection.items.is_empty() {
            return Err(TestSupportError::expectation(format!(
                "expected empty selection before bounding-box press; found {selection:?}"
            )));
        }
        with_state(|state| {
            state.points.push(point(px(centre.x), px(centre.y)));
        });
        support::set_scenario_data(ScenarioData {
            shape_id,
            selection_after_press: None,
        });
        Ok(())
    })
}

#[when("the centre of the square is clicked")]
fn centre_of_square_is_clicked(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let centre = require_point(0, "square centre")?;
    with_visual_cx(cx, |visual_cx, view| {
        visual_cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        support::with_mut_scenario_data::<ScenarioData, _>("bounding-box press", |data| {
            data.selection_after_press = Some(selection);
        })?;
        visual_cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("only the square is selected")]
fn only_square_is_selected() -> Result<(), TestSupportError> {
    let (shape_id, selection_after_press) =
        support::with_scenario_data::<ScenarioData, _>("bounding-box selection", |data| {
            (data.shape_id, data.selection_after_press.clone())
        })?;
    let selection = selection_after_press.ok_or_else(|| {
        TestSupportError::missing("selection after press", "recorded by the click step")
    })?;
    let expected = vec![SelItem::Shape(shape_id)];
    if selection.items != expected {
        return Err(TestSupportError::expectation(format!(
            "expected only square {shape_id:?}; selection={selection:?}"
        )));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/selection.feature",
    name = "Clicking inside a shape bounding box selects it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn select_shape_by_bbox(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
