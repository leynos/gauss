//! BDD coverage for clearing selection by clicking empty canvas space.
//!
//! This binary binds the corresponding scenario in `selection.feature` to the
//! GPUI `GpuiHarness`. It combines canvas utilities from `common` with reusable
//! lifecycle state in `selection_bdd::support`. The assertion reads the Phase
//! 0 shell directly; model-only helpers shared by other binaries live in
//! `test_support::selection`.

mod common;
#[path = "selection_bdd/support.rs"]
pub mod support;

use common::{canvas_bounds, click_left_and_wait, read_selection};
use gauss::model::{Document, SelItem, Selection, ShapeId, Vec2};
use gauss_core::test_helpers::square_shape;
use gpui::{TestAppContext, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioContext, ScenarioStateCleanup, require_point, with_state, with_visual_cx};
use test_support::TestSupportError;

struct ScenarioData {
    selected_shape_id: ShapeId,
}

#[given("a selected square is arranged")]
fn selected_square_is_arranged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_visual_cx(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let mut document = Document::new();
        let shape_id = document.append_shape(square_shape(
            ShapeId::default(),
            origin.add(Vec2::new(10.0, 10.0)),
            origin.add(Vec2::new(60.0, 60.0)),
        ));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(Selection {
                    items: vec![SelItem::Shape(shape_id)],
                });
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        let selection = read_selection(visual_cx, view);
        let expected_selection = Selection {
            items: vec![SelItem::Shape(shape_id)],
        };
        if selection != expected_selection {
            return Err(TestSupportError::expectation(format!(
                "expected selected square before empty-canvas click; found {selection:?}"
            )));
        }
        let click_x = (bounds.size.width - px(2.0)).max(px(2.0));
        let click_y = (bounds.size.height - px(2.0)).max(px(2.0));
        with_state(|state| {
            state
                .points
                .push(point(bounds.origin.x + click_x, bounds.origin.y + click_y));
        });
        support::set_scenario_data(ScenarioData {
            selected_shape_id: shape_id,
        });
        Ok(())
    })
}

#[when("empty canvas space is clicked")]
fn empty_canvas_space_is_clicked(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point = require_point(0, ScenarioContext::EmptyCanvasPoint)?;
    with_visual_cx(cx, |visual_cx, _view| {
        click_left_and_wait(visual_cx, point);
        Ok(())
    })
}

#[then("the selection is empty")]
fn selection_is_empty(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let selected_shape_id =
        support::with_scenario_data::<ScenarioData, _>(ScenarioContext::SelectedSquare, |data| {
            data.selected_shape_id
        })?;
    with_visual_cx(cx, |visual_cx, view| {
        let selection = read_selection(visual_cx, view);
        if !selection.items.is_empty() {
            return Err(TestSupportError::expectation(format!(
                "expected square {selected_shape_id:?} to be cleared; selection={selection:?}"
            )));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/selection.feature",
    name = "Clicking empty space clears selection",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn clear_selection(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
