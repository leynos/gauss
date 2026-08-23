//! Harness facade for the `tests/gpui_shell_style_controls.rs` integration test.
//!
//! Re-exports canvas interaction, document history, and shape inspection for
//! style-control scenarios.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod canvas_click;
mod document;
mod document_undo;
mod draw_shape;
mod escape_input;
mod history;

/// Maps an anchor into a canvas point for GPUI input events.
pub use anchor_point::anchor_to_canvas_point;
/// Looks up the rendered bounds of the test canvas.
pub use canvas_bounds::canvas_bounds;
/// Clicks the canvas at a position and drains the event loop.
pub use canvas_click::click_canvas_and_wait;
/// Returns an owned snapshot of the shell view's current document.
pub use document::read_document;
/// Dispatches document undo and drains the event loop.
pub use document_undo::simulate_document_undo;
/// Returns the first drawn shape other than the initial demo shape.
pub use draw_shape::require_draw_shape;
/// Dispatches an unmodified Escape key-down event and drains the event loop.
pub use escape_input::simulate_escape;
/// Returns the current number of entries in the view's document history.
pub use history::read_history_len;
