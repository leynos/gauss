//! Harness facade for the `tests/gpui_selection_multi_select.rs` integration test.
//!
//! Re-exports drawing and canvas setup plus modifier input for multi-selection scenarios.

#[path = "anchor_point.rs"]
mod anchor_point;
#[path = "canvas.rs"]
mod canvas;
#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "document.rs"]
mod document;
#[path = "draw_point.rs"]
mod draw_point;
#[path = "draw_shape.rs"]
mod draw_shape;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "modifiers.rs"]
mod modifiers;

/// Maps selection anchors into the window coordinates used by GPUI input.
pub use anchor_point::anchor_to_canvas_point;
/// Returns the rendered canvas bounds used to position selection input.
pub use canvas_bounds::canvas_bounds;
/// Returns an owned document snapshot for multi-selection setup assertions.
pub use document::read_document;
/// Adds a draw point at a canvas position and parks the GPUI event loop.
pub use draw_point::draw_point;
/// Returns the first non-demo shape for multi-selection assertions.
pub use draw_shape::require_draw_shape;
/// Initializes Gauss in the GPUI test application context.
pub use init_app::init_test_app;
/// Performs the initial draw and parks the GPUI event loop before input.
pub use initial_draw::ensure_initial_draw;
/// Enables the secondary Shift modifier for multi-selection input.
pub use modifiers::shift_secondary;
