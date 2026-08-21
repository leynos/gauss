//! BDD coverage for drawing-mode activation and stale-path recovery.

#[path = "common/scenario_state.rs"]
mod scenario_state;

#[path = "common/gpui_history_draw_undo.rs"]
mod common;
#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open.rs"]
mod history_bdd_support_open;

use common::{canvas_bounds, click_canvas_and_wait, read_document, require_draw_shape};
use gauss::model::ShapeId;
use gauss::ui::GpuiActivatePenTool;
use gpui::{Pixels, Point, TestAppContext, point, px};
use history_bdd_support::{DurableShell, missing};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

#[derive(Default)]
struct DrawState {
    shell: Option<DurableShell>,
    click: Option<Point<Pixels>>,
    shape_count_before: Option<usize>,
    stale_path: Option<ShapeId>,
    new_shape: Option<ShapeId>,
}

crate::scenario_state!(DrawState);

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| missing("Phase 0 shell"))
}

#[given("a fresh Phase 0 shell window in manipulate mode")]
fn shell_in_manipulate_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (click, count) = shell.with_visual(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let click = point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0));
        visual_cx.update(|_window, app| {
            view.update(app, |phase0, view_cx| {
                phase0.enter_manipulate_mode_for_tests();
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        Ok((click, read_document(visual_cx, view).len()))
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.click = Some(click);
        state.shape_count_before = Some(count);
    });
    Ok(())
}

#[given("a fresh Phase 0 shell window with a stale active draw path")]
fn shell_with_stale_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let stale_path = ShapeId::from_accesskit_node_id(9_999);
    let click = shell.with_visual(cx, |visual_cx, view| {
        let bounds = canvas_bounds(visual_cx)?;
        let click = point(bounds.origin.x + px(12.0), bounds.origin.y + px(12.0));
        visual_cx.update(|_window, app| {
            view.update(app, |phase0, view_cx| {
                phase0.set_draw_active_shape_for_tests(Some(stale_path));
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        let active = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
        if active != Some(stale_path) {
            return Err(TestSupportError::expectation(
                "expected test setup to install a stale active path",
            ));
        }
        Ok(click)
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.click = Some(click);
        state.stale_path = Some(stale_path);
    });
    Ok(())
}

#[when("the canvas is clicked in manipulate mode")]
fn click_canvas_in_manipulate_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_canvas(cx)
}

#[when("the canvas is clicked in draw mode")]
fn click_canvas_in_draw_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_canvas(cx)
}

fn click_canvas(cx: &mut TestAppContext) -> Result<(), TestSupportError> {
    let shell = shell()?;
    let click = with_state(|state| state.click).ok_or_else(|| missing("canvas click"))?;
    shell.with_visual(cx, |visual_cx, view| {
        click_canvas_and_wait(visual_cx, click);
        if with_state(|state| state.stale_path).is_some() {
            let document = read_document(visual_cx, view);
            let shape = require_draw_shape(&document, "after stale active path click")?;
            with_state(|state| state.new_shape = Some(shape.id));
        }
        Ok(())
    })
}

#[when("the pen tool is activated")]
fn activate_pen_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shell = shell()?;
    shell.with_visual(cx, |visual_cx, _view| {
        visual_cx.dispatch_action(GpuiActivatePenTool);
        visual_cx.run_until_parked();
        let bounds = canvas_bounds(visual_cx)?;
        with_state(|state| {
            state.click = Some(point(
                bounds.origin.x + px(24.0),
                bounds.origin.y + px(24.0),
            ));
        });
        Ok(())
    })
}

fn expected_shape_count() -> Result<usize, TestSupportError> {
    with_state(|state| state.shape_count_before).ok_or_else(|| missing("initial shape count"))
}

#[then("the document shape count is unchanged")]
fn shape_count_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_shape_count(cx, expected_shape_count()?)
}

#[then("the document shape count has gained 1 shape")]
fn shape_count_gained_one(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_shape_count(cx, expected_shape_count()?.saturating_add(1))
}

fn assert_shape_count(cx: &mut TestAppContext, expected: usize) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = read_document(visual_cx, view).len();
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "expected document shape count {expected}, found {actual}"
            )));
        }
        Ok(())
    })
}

#[then("a new open shape has 1 anchor and 0 segments")]
fn new_open_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after stale active path click")?;
        if shape.path.anchors.len() != 1 || !shape.path.segments.is_empty() {
            return Err(TestSupportError::expectation(format!(
                "expected one anchor and no segments, found {} anchor(s) and {} segment(s)",
                shape.path.anchors.len(),
                shape.path.segments.len()
            )));
        }
        Ok(())
    })
}

#[then("the active draw path tracks the new shape")]
fn active_path_tracks_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| state.new_shape).ok_or_else(|| missing("new shape"))?;
    let stale = with_state(|state| state.stale_path).ok_or_else(|| missing("stale path"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
        if actual != Some(expected) || actual == Some(stale) {
            return Err(TestSupportError::expectation(format!(
                "expected active path {expected:?} to replace stale path {stale:?}, found {actual:?}"
            )));
        }
        Ok(())
    })
}

#[then("no history error is present")]
fn no_history_error(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        let error = visual_cx.read(|app| {
            view.read(app)
                .last_history_error_for_tests()
                .map(str::to_owned)
        });
        if error.is_some() {
            return Err(TestSupportError::expectation(format!(
                "expected no history error, found {error:?}"
            )));
        }
        Ok(())
    })
}

#[scenario(
    path = "tests/features/history_draw_undo.feature",
    name = "Activating the pen tool from manipulate mode allows drawing",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn activate_pen_tool_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/history_draw_undo.feature",
    name = "Drawing recovers from a stale active path",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn stale_path_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
