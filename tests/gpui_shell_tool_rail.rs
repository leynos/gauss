//! Behavioural coverage for the shell tool rail through `GpuiHarness`.

#[path = "shell_bdd/click.rs"]
mod click_support;
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
use gpui::TestAppContext;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{fresh_shell_with, with_shell, ScenarioStateCleanup};
use test_support::{TestSupportError, TestSupportResult};

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell_with(cx, Phase0Shell::new)
}

#[given("a fresh Phase 0 shell window with an active draw shape")]
fn shell_with_active_draw_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell_with(cx, Phase0Shell::new)?;
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                let demo_id = shell.document().shape_id_at(0).ok_or_else(|| {
                    TestSupportError::missing("demo shape", "initial shell document")
                })?;
                shell.set_draw_active_shape_for_tests(Some(demo_id));
                Ok::<(), TestSupportError>(())
            })
        })?;
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("the Select tool is clicked")]
fn click_select_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_tool(cx, "#tool-select")
}

#[when("the Curve tool is clicked")]
fn click_curve_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_tool(cx, "#tool-draw-curve")
}

#[when("the Line tool is clicked")]
fn click_line_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    click_tool(cx, "#tool-draw-line")
}

fn click_tool(cx: &mut TestAppContext, selector: &'static str) -> TestSupportResult<()> {
    with_shell(cx, |visual_cx, _view| click_selector(visual_cx, selector))
}

#[then("manipulate mode is active")]
fn manipulate_mode_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_mode(cx, "Mode: Manipulate")
}

#[then("Bezier auto draw mode is active")]
fn bezier_auto_draw_mode_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_mode(cx, "Mode: Draw (Bezier (auto))")
}

#[then("line draw mode is active")]
fn line_draw_mode_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_mode(cx, "Mode: Draw (Line)")
}

fn expect_mode(cx: &mut TestAppContext, expected: &str) -> TestSupportResult<()> {
    with_shell(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
        expect_equal(&actual.as_str(), &expected, "active tool mode")
    })
}

#[then("no draw shape is active")]
fn no_draw_shape_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let active_shape = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
        expect_true(active_shape.is_none(), "expected no active draw shape")
    })
}

#[scenario(
    path = "tests/features/shell_tool_rail.feature",
    name = "Select tool enters manipulate mode and clears active shape",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn select_tool_enters_manipulate_mode(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(
    path = "tests/features/shell_tool_rail.feature",
    name = "Draw tools switch edge modes",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn draw_tools_switch_edge_modes(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
