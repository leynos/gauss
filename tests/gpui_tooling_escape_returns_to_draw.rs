//! GPUI behavioural scenarios for Escape mode transitions.

#[path = "common/gpui_tooling_escape_returns_to_draw.rs"]
mod common;

#[path = "tooling_bdd/escape_returns_steps.rs"]
mod escape_returns_steps;

#[path = "tooling_bdd/state.rs"]
mod state;

#[scenario(
    path = "tests/features/tooling_escape_returns_to_draw.feature",
    name = "Escape in manipulate mode returns to draw mode",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn escape_returns_to_draw(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/tooling_escape_returns_to_draw.feature",
    name = "Escape cancels a manipulate drag preview without a history commit",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn escape_cancels_drag_preview(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
