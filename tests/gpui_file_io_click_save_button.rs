//! Behavioural coverage for opening the save prompt through the GPUI Save button.

mod common;

use std::cell::RefCell;

use common::{ensure_initial_draw, file_io::DurableShell, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, TestAppContext, point, px};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

thread_local! {
    static SHELL: RefCell<Option<DurableShell>> = const { RefCell::new(None) };
}

fn reset_state() {
    SHELL.with(|cell| *cell.borrow_mut() = None);
}

struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state();
    }
}

#[fixture]
fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state();
    ScenarioStateCleanup
}

fn shell() -> Result<DurableShell, TestSupportError> {
    SHELL
        .with(|cell| cell.borrow().clone())
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
    SHELL.with(|cell| *cell.borrow_mut() = Some(DurableShell::new(entity, visual_cx)));
    shell()?;
    Ok(())
}

#[then("no file path prompt is visible")]
fn no_file_path_prompt(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(
            "expected no file path prompt to be visible",
        ));
    }
    Ok(())
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
    if !cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(
            "expected a file path prompt to be visible",
        ));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/file_io_click_save_button.feature",
    name = "Clicking Save opens the save prompt",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn clicking_save_opens_prompt(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
