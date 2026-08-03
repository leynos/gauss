//! Escape-key simulation for GPUI integration tests.

use gpui::{KeyDownEvent, Keystroke, Modifiers, VisualTestContext};

/// Dispatches an unmodified Escape key-down event and drains the event loop.
pub fn simulate_escape(visual_cx: &mut VisualTestContext) {
    visual_cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers::none(),
            key: "escape".to_owned(),
            key_char: None,
        },
        is_held: false,
    });
    visual_cx.run_until_parked();
}
