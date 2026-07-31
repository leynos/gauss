//! Behavioural coverage for opening the save prompt through the GPUI Save button.

mod common;
#[path = "common/file_io.rs"]
mod file_io;
#[path = "common/scenario_state.rs"]
mod scenario_state;

use common::{ensure_initial_draw, init_test_app};
use file_io::{DurableShell, assert_no_path_prompt, assert_path_prompt};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, TestAppContext, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

crate::scenario_state!(Option<DurableShell>);

/// Clone the durable shell handle out of thread-local scenario state.
///
/// # Errors
///
/// Returns `Err` if the Given step that populates the handle has not run yet.
fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.clone())
        .ok_or_else(|| TestSupportError::missing("shell handles", "set by the Given step"))
}

#[given("a fresh Phase 0 shell window for file I/O")]
fn fresh_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    with_state(|state| *state = Some(DurableShell::new(entity, visual_cx)));
    shell()?;
    Ok(())
}

#[then("no file path prompt is visible")]
fn no_file_path_prompt(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_no_path_prompt(cx, "expected no file path prompt to be visible")
}

#[when("the Save button is clicked")]
fn click_save_button(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual_cx(cx, |visual_cx, _entity| {
        let bounds = visual_cx.debug_bounds("#save-button").ok_or_else(|| {
            TestSupportError::missing("Save button bounds", "after the initial draw")
        })?;
        let position = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
        visual_cx.simulate_mouse_move(position, None, Modifiers::none());
        visual_cx.simulate_click(position, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })?;
    cx.run_until_parked();
    Ok(())
}

#[then("a file path prompt is visible")]
fn file_path_prompt(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_path_prompt(cx, "expected a file path prompt to be visible")
}

#[scenario(
    path = "tests/features/file_io_click_save_button.feature",
    name = "Clicking Save opens the save prompt",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn clicking_save_opens_prompt(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
