//! Harness facade for the `tests/gpui_tooling_toggle_segment_kind.rs` integration test.
//!
//! Re-exports canvas input, document history, shape lookup, and vector assertions
//! for segment-kind toggles.

#[path = "anchor_point.rs"]
mod anchor_point;
#[path = "canvas.rs"]
mod canvas;
#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "canvas_click.rs"]
mod canvas_click;
#[path = "document.rs"]
mod document;
#[path = "document_undo.rs"]
mod document_undo;
#[path = "draw_shape.rs"]
mod draw_shape;
#[path = "escape_input.rs"]
mod escape_input;
#[path = "history.rs"]
mod history;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "vec2_assertion.rs"]
mod vec2_assertion;

/// Converts a document or absolute anchor into a GPUI canvas input point.
pub use anchor_point::anchor_to_canvas_point;
/// Keeps simulated canvas input two pixels inside the rendered bounds.
pub use canvas::CANVAS_PADDING_PX;
/// Looks up the rendered `#phase0-canvas` bounds for harness input.
pub use canvas_bounds::canvas_bounds;
/// Clicks a canvas point without modifiers and waits for the event loop.
pub use canvas_click::click_canvas_and_wait;
/// Provides the harness with an owned snapshot of the shell's current document.
pub use document::read_document;
/// Dispatches document undo and waits for the GPUI event loop to become parked.
pub use document_undo::simulate_document_undo;
/// Finds the first drawn shape other than the document's initial demo shape.
pub use draw_shape::require_draw_shape;
/// Dispatches an unmodified Escape key event and waits for the event loop.
pub use escape_input::simulate_escape;
/// Provides the harness with the shell's current document-history entry count.
pub use history::read_history_len;
/// Initializes the Gauss application in the harness's GPUI test context.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to become parked.
pub use initial_draw::ensure_initial_draw;
/// Checks that harness-observed vectors are within the helper's tolerance.
pub use vec2_assertion::assert_vec2_close;
