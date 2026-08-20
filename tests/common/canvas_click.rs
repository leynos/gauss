//! Canvas-click simulation for GPUI integration tests.

use gpui::{Modifiers, Pixels, Point, VisualTestContext};

/// Moves to `position`, clicks without modifiers, and drains the event loop.
///
/// This infallible helper returns only after the GPUI test context is parked.
pub fn click_canvas_and_wait(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
    visual_cx.run_until_parked();
}
