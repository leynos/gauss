//! GPUI behavioural scenario for toggling and undoing a selected segment.

#[path = "common/gpui_tooling_toggle_segment_kind.rs"]
mod common;

#[path = "tooling_bdd/state.rs"]
mod state;

#[path = "tooling_bdd/toggle_segment_steps.rs"]
mod toggle_segment_steps;

#[scenario(
    path = "tests/features/tooling_toggle_segment_kind.feature",
    name = "Tab toggles the selected segment kind and undo restores it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn toggle_segment_and_undo(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
