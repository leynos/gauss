//! Shared keyboard-action step definitions for GPUI tooling scenarios.

use crate::{common, state};
use gpui::TestAppContext;
use rstest_bdd_macros::when;
use test_support::TestSupportError;

/// Connects the scenario binary's marker to its typed scratch data.
pub(crate) trait SharedStepStateMarker {
    /// The scenario-specific state held while shared steps run.
    type Data: 'static;
}

#[when("the draw edge mode is switched to Bezier auto")]
fn switch_to_bezier_auto(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx::<<crate::SharedStepState as SharedStepStateMarker>::Data, _>(
        cx,
        |visual_cx, _view, _data| {
            visual_cx.simulate_keystrokes("tab");
            visual_cx.run_until_parked();
            Ok(())
        },
    )
}

#[when("Escape is pressed")]
fn press_escape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx::<<crate::SharedStepState as SharedStepStateMarker>::Data, _>(
        cx,
        |visual_cx, _view, _data| {
            common::simulate_escape(visual_cx);
            Ok(())
        },
    )
}
