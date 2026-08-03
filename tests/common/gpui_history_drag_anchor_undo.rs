//! Support facade for the `gpui_history_drag_anchor_undo` integration test.
//!
//! It exposes app lifecycle, drawing, document-history, anchor-drag, Escape,
//! and vector-assertion helpers required by that harness.

mod canvas;
mod canvas_bounds;
mod canvas_drag_anchor;
mod canvas_drag_values;
mod document;
mod document_undo;
mod draw_point;
mod draw_shape;
mod escape_input;
mod history;
mod init_app;
mod initial_draw;
mod vec2_assertion;

/// Anchor-drag points and displacement used by this history harness.
pub use canvas_drag_anchor::CanvasDragScenario;
/// Builds the bounded anchor-drag scenario for this history harness.
pub use canvas_drag_anchor::canvas_drag_scenario;
/// Takes an owned document snapshot for before-and-after comparisons.
pub use document::read_document;
/// Dispatches document undo and drains the event loop.
pub use document_undo::simulate_document_undo;
/// Adds a draw point and waits for input processing to settle.
pub use draw_point::draw_point;
/// Requires the non-demo shape produced by the drawing phase.
pub use draw_shape::require_draw_shape;
/// Dispatches Escape and waits for the mode transition to settle.
pub use escape_input::simulate_escape;
/// Reads the current document-history length from the shell view.
pub use history::read_history_len;
/// Initializes Gauss for this GPUI integration harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
/// Checks anchor positions using the shared vector tolerance.
pub use vec2_assertion::assert_vec2_close;
