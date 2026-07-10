//! GPUI behavioural scenarios for Phase 0 path closing.

#[path = "tooling_bdd/close_path_steps.rs"]
mod close_path_steps;
#[path = "common/gpui_tooling_close_path.rs"]
mod common;
#[path = "tooling_bdd/state.rs"]
mod state;

use rstest_bdd_macros::scenario;
use serial_test::serial;
use state::{ScenarioStateCleanup, scenario_state_cleanup};

#[scenario(
    path = "tests/features/tooling_close_path.feature",
    name = "Clicking near the first anchor closes the path and enters manipulate mode",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn clicking_near_first_anchor(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/tooling_close_path.feature",
    name = "Closing in Bezier mode uses a cubic closing segment",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn closing_in_bezier_mode(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/tooling_close_path.feature",
    name = "Clicking the first anchor before a third point keeps the path open",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn early_first_anchor_click(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
