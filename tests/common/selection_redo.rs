//! Selection-redo dispatch for GPUI integration tests.

use gauss::ui::GpuiSelectionRedo;
use gpui::VisualTestContext;

/// Dispatches `GpuiSelectionRedo` and drains events until completion.
pub fn simulate_selection_redo(visual_cx: &mut VisualTestContext) {
    visual_cx.dispatch_action(GpuiSelectionRedo);
    visual_cx.run_until_parked();
}
