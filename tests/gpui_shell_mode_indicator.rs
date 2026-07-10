//! Behavioural coverage for the shell mode indicator through `GpuiHarness`.

#[path = "common/gpui_shell_mode_indicator.rs"]
mod common;

#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;

#[path = "shell_bdd/support.rs"]
mod support;

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell_with(cx, Phase0Shell::new);
    with_shell(cx, |_visual_cx, _view| Ok(()))?;
    Ok(())
}

#[when("the edge mode is cycled with Tab")]
fn cycle_edge_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        visual_cx.simulate_keystrokes("tab");
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("manipulate mode is entered")]
fn enter_manipulate_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell: &mut Phase0Shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the mode indicator reads {expected}")]
fn mode_indicator_reads(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    expected: String,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
        let expected_value = String::from(expected.trim_matches('"'));
        expect_equal(&actual, &expected_value, "mode indicator")
    })
}

#[scenario(
    path = "tests/features/shell_mode_indicator.feature",
    name = "Mode indicator follows tool and edge mode",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn mode_indicator_follows_tool_and_edge_mode(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
