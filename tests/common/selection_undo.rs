//! Selection-undo dispatch for GPUI integration tests.

use gauss::ui::GpuiSelectionUndo;
use gpui::VisualTestContext;

pub fn simulate_selection_undo(visual_cx: &mut VisualTestContext) {
    visual_cx.dispatch_action(GpuiSelectionUndo);
    visual_cx.run_until_parked();
}
