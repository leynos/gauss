//! Behavioural shell-chrome coverage through `GpuiHarness`.

#[path = "shell_bdd/click.rs"]
mod click_support;
mod common;
#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;
#[path = "shell_bdd/expect_true.rs"]
mod expect_true_support;
#[path = "shell_bdd/lifecycle.rs"]
mod lifecycle;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "shell_bdd/support.rs"]
mod support;

use click_support::click_selector;
use expect_equal_support::expect_equal;
use expect_true_support::expect_true;
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, fresh_shell_with, with_shell};
use test_support::TestSupportError;

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell_with(cx, Phase0Shell::new)
}

#[given("a fresh testable Phase 0 shell window")]
fn fresh_testable_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell_with(cx, Phase0Shell::new_for_tests)
}

#[when("the canvas is clicked")]
fn click_canvas(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        common::click_canvas_and_wait(
            visual_cx,
            point(bounds.origin.x + px(4.0), bounds.origin.y + px(4.0)),
        );
        Ok(())
    })
}

#[when("two anchors are placed")]
fn place_two_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        common::click_canvas_and_wait(
            visual_cx,
            point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0)),
        );
        common::click_canvas_and_wait(
            visual_cx,
            point(bounds.origin.x + px(80.0), bounds.origin.y + px(24.0)),
        );
        Ok(())
    })
}

macro_rules! button_step {
    ($name:ident, $phrase:literal, $selector:literal) => {
        #[when($phrase)]
        fn $name(
            #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
        ) -> Result<(), TestSupportError> {
            with_shell(cx, |visual_cx, _view| click_selector(visual_cx, $selector))
        }
    };
}

button_step!(click_open, "the Open button is clicked", "#open-button");
button_step!(click_save, "the Save button is clicked", "#save-button");
button_step!(click_undo, "the Undo button is clicked", "#undo-button");
button_step!(click_redo, "the Redo button is clicked", "#redo-button");
button_step!(click_quit, "the Quit button is clicked", "#quit-button");

#[then("the shell records the canvas click")]
fn shell_records_canvas_click(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let click = visual_cx.read(|app| view.read(app).last_canvas_click_screen());
        expect_true(click.is_some(), "expected shell to record canvas click")
    })
}

#[then("a new-path prompt is requested")]
fn new_path_prompt_is_requested(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    cx.run_until_parked();
    expect_true(cx.did_prompt_for_new_path(), "expected a new-path prompt")?;
    cx.simulate_new_path_selection(|_directory| None);
    cx.run_until_parked();
    Ok(())
}

#[then("the draw shape anchor count is {count:usize}")]
fn draw_shape_anchor_count(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: usize,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let document = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&document, "chrome button scenario")?;
        expect_equal(&shape.path.anchors.len(), &count, "draw shape anchor count")
    })
}

#[then("the shell requests quit")]
fn shell_requests_quit(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_true(
        shell_did_request_quit!(cx)?,
        "expected shell to request quit",
    )
}

#[scenario(path = "tests/features/shell_chrome.feature", name = "Canvas input remains active beneath chrome", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn canvas_input_remains_active(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(path = "tests/features/shell_chrome.feature", name = "Open button requests a path", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn open_button_requests_path(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(path = "tests/features/shell_chrome.feature", name = "Save button requests a path", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn save_button_requests_path(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(path = "tests/features/shell_chrome.feature", name = "Undo and redo buttons update the document", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn undo_redo_buttons_update_document(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(path = "tests/features/shell_chrome.feature", name = "Quit button requests quit", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn quit_button_requests_quit(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
