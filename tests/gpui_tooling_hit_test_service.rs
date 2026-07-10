//! GPUI behavioural scenarios for shared hit-test service wiring.

#[path = "common/gpui_tooling_hit_test_service.rs"]
mod common;

#[path = "tooling_bdd/hit_test_steps.rs"]
mod hit_test_steps;

#[path = "tooling_bdd/state.rs"]
mod state;
#[scenario(path = "tests/features/tooling_hit_test_service.feature", name = "Hovering a handle prefers the handle hit", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn hovering_handle_prefers_handle_hit(
    #[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(path = "tests/features/tooling_hit_test_service.feature", name = "The hover hit clears when the cursor moves to empty space", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn hover_clears_in_empty_space(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(path = "tests/features/tooling_hit_test_service.feature", name = "The hover hit clears when leaving manipulate mode", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn hover_clears_outside_manipulate(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {
}

#[scenario(path = "tests/features/tooling_hit_test_service.feature", name = "Clicking overlapping shapes selects the topmost shape", harness = rstest_bdd_harness_gpui::GpuiHarness)]
#[serial]
fn overlapping_shapes_select_topmost(
    #[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
