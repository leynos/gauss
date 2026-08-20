//! Document-redo dispatch for GPUI integration tests.

use gauss::ui::GpuiRedo;
use gpui::VisualTestContext;

/// Dispatches `GpuiRedo` and drains the event loop to complete document redo.
pub fn simulate_document_redo(visual_cx: &mut VisualTestContext) {
    visual_cx.dispatch_action(GpuiRedo);
    visual_cx.run_until_parked();
}
