//! Draw-point input for GPUI integration tests.

use gpui::{Modifiers, Pixels, Point, VisualTestContext};

/// Moves to `position`, clicks to add a draw point, and drains the event loop.
pub fn draw_point(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
    visual_cx.run_until_parked();
}
