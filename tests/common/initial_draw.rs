//! Initial drawing for GPUI integration tests.

use gpui::VisualTestContext;

pub fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| {
        let _draw = window.draw(app);
    });
    visual_cx.run_until_parked();
}
