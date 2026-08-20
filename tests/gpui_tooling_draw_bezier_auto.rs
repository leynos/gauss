//! GPUI behavioural scenario for Bezier-auto drawing.

#[path = "common/gpui_tooling_draw_bezier_auto.rs"]
mod common;
#[path = "tooling_bdd/draw_bezier_auto_steps.rs"]
mod draw_bezier_auto_steps;
#[path = "tooling_bdd/shared_steps.rs"]
mod shared_steps;
#[path = "tooling_bdd/state.rs"]
mod state;

use rstest_bdd_macros::scenario;
use serial_test::serial;
use state::{ScenarioStateCleanup, scenario_state_cleanup};

struct SharedStepState;

impl shared_steps::SharedStepStateMarker for SharedStepState {
    type Data = draw_bezier_auto_steps::BezierAutoState;
}

#[scenario(
    path = "tests/features/tooling_draw_bezier_auto.feature",
    name = "Tab switches to Bezier auto and synthesizes handles",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn draw_bezier_auto(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
