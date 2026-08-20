//! Left-button click simulation for GPUI integration tests.

use gpui::{Modifiers, MouseButton, Pixels, Point, VisualTestContext};

/// Performs a left-button click at `position` and drains the event loop.
pub fn click_left_and_wait(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    visual_cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
}
