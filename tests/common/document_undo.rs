//! Document-undo dispatch for GPUI integration tests.

use gauss::ui::GpuiUndo;
use gpui::VisualTestContext;

/// Dispatches `GpuiUndo` and drains the event loop to complete document undo.
pub fn simulate_document_undo(visual_cx: &mut VisualTestContext) {
    visual_cx.dispatch_action(GpuiUndo);
    visual_cx.run_until_parked();
}
