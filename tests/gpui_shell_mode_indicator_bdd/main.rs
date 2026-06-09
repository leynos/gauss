//! GPUI behavioural test for the Phase 0 mode indicator, expressed as a
//! Gherkin scenario driven by the `rstest-bdd` 0.6.0 GPUI harness.
//!
//! This replaces the raw `#[gpui::test]` form previously in
//! `tests/gpui_shell_mode_indicator.rs`. The scenario asserts the mode
//! indicator reads `Mode: Draw (Line)` initially, `Mode: Draw (Bezier (auto))`
//! after `Tab` toggles the edge mode, and `Mode: Manipulate` after switching
//! to manipulate mode.
//!
//! The harness injects `gpui::TestAppContext` under the reserved fixture key
//! `rstest_bdd_harness_context`; the GPUI attribute policy
//! (`#[rstest::rstest]` + `#[gpui::test]`) is inferred from the canonical
//! `rstest_bdd_harness_gpui::GpuiHarness` path, so `attributes = ...` is
//! omitted. `#[serial]` is required because GPUI scenarios share the
//! thread-local durable-handle state and a process-wide test app slot.

mod steps;

use rstest_bdd_macros::scenario;
use serial_test::serial;

#[scenario(
    path = "tests/features/shell_mode_indicator.feature",
    name = "Mode indicator reflects tool and edge mode",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn scenario_mode_indicator_reflects_tool_and_edge_mode(
    #[from(steps::scenario_state_cleanup)] _cleanup: steps::ScenarioStateCleanup,
) {
}
