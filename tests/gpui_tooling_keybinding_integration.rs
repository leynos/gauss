//! GPUI behavioural scenarios for keybinding registration and dispatch.

#[path = "common/gpui_tooling_keybinding_integration.rs"]
mod common;
#[path = "tooling_bdd/keybinding_steps.rs"]
mod keybinding_steps;
#[path = "tooling_bdd/state.rs"]
mod state;

use rstest_bdd_macros::scenario;
use serial_test::serial;
use state::{ScenarioStateCleanup, scenario_state_cleanup};

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "UI initialization registers action bindings", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn ui_init_registers_bindings(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Select all selects every shape", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn select_all(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Deselect all clears the selection", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn deselect_all(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Activating the pen tool switches to draw mode", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn activate_pen_tool(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Activating the select tool switches to manipulate mode", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn activate_select_tool(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Activating the select tool clears the active draw shape", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn select_tool_clears_active_shape(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {
}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Tab toggles the edge mode in draw mode", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn tab_toggles_edge_mode(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_keybinding_integration.feature", name = "Tab does not toggle the draw edge mode in manipulate mode", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn tab_is_noop_in_manipulate_mode(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
