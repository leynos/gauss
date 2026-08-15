//! Behavioural coverage for the shell Quit button through `GpuiHarness`.

#[path = "shell_bdd/click.rs"]
mod click_support;
#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "shell_bdd/expect_true.rs"]
mod expect_true_support;
#[path = "shell_bdd/lifecycle.rs"]
mod lifecycle;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "shell_bdd/support.rs"]
mod support;

use click_support::click_selector;
use expect_true_support::expect_true;
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{fresh_shell_with, with_shell, ScenarioStateCleanup};
use test_support::TestSupportError;

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell_with(cx, Phase0Shell::new)
}

#[when("the Quit button is clicked")]
fn click_quit_button(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        click_selector(visual_cx, "#quit-button")
    })
}

#[then("the shell requests quit")]
fn shell_requests_quit(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_true(
        shell_did_request_quit!(cx)?,
        "expected the shell to request quit",
    )
}

#[scenario(
    path = "tests/features/shell_quit.feature",
    name = "Quit button requests quit",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn quit_button_requests_quit(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
