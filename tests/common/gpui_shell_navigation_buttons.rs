//! Harness facade for the `tests/gpui_shell_navigation_buttons.rs` integration test.
//!
//! Re-exports drawing, canvas geometry, keyboard escape, and modifier input for
//! navigation-button scenarios.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod document;
mod draw_point;
mod draw_shape;
mod escape_input;
mod modifiers;

/// Maps an anchor into a canvas point for GPUI input events.
pub use anchor_point::anchor_to_canvas_point;
/// Two-pixel inset used to keep test input points inside the canvas bounds.
pub use canvas::CANVAS_PADDING_PX;
/// Looks up the rendered bounds of the test canvas.
pub use canvas_bounds::canvas_bounds;
/// Returns an owned snapshot of the shell view's current document.
pub use document::read_document;
/// Clicks the canvas to add a draw point and drains the event loop.
pub use draw_point::draw_point;
/// Returns the first drawn shape other than the initial demo shape.
pub use draw_shape::require_draw_shape;
/// Dispatches an unmodified Escape key-down event and drains the event loop.
pub use escape_input::simulate_escape;
/// Returns `modifiers` with the secondary Shift modifier enabled.
pub use modifiers::shift_secondary;
