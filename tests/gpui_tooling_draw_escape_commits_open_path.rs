//! GPUI behavioural scenario for committing an open path with Escape.

#[path = "common/gpui_tooling_draw_escape_commits_open_path.rs"]
mod common;

#[path = "tooling_bdd/draw_escape_steps.rs"]
mod draw_escape_steps;

#[path = "tooling_bdd/state.rs"]
mod state;

#[scenario(
    path = "tests/features/tooling_draw_escape_commits_open_path.feature",
    name = "Escape commits an open path and enters manipulate mode",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn escape_commits_open_path(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
