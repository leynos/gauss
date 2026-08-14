//! GPUI behavioural scenarios for Escape mode transitions.

#[path = "common/gpui_tooling_escape_returns_to_draw.rs"]
mod common;
#[path = "tooling_bdd/escape_returns_steps.rs"]
mod escape_returns_steps;
#[path = "tooling_bdd/shared_steps.rs"]
mod shared_steps;
#[path = "tooling_bdd/state.rs"]
mod state;

use rstest_bdd_macros::scenario;
use serial_test::serial;
use state::{ScenarioStateCleanup, scenario_state_cleanup};

struct SharedStepState;

impl shared_steps::SharedStepStateMarker for SharedStepState {
    type Data = escape_returns_steps::EscapeState;
}

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
