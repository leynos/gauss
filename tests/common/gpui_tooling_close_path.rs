//! Harness facade for the `tests/gpui_tooling_close_path.rs` integration test.
//!
//! Re-exports app and canvas setup, drawing input, and shape inspection for
//! close-path tooling scenarios.

#[path = "anchor_point.rs"]
mod anchor_point;
#[path = "canvas.rs"]
mod canvas;
#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "draw_point.rs"]
mod draw_point;
#[path = "draw_shape.rs"]
mod draw_shape;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;

/// Maps close-path anchors into the window coordinates used by GPUI input.
pub use anchor_point::anchor_to_canvas_point;
/// Returns the rendered canvas bounds used to position close-path input.
pub use canvas_bounds::canvas_bounds;
/// Adds a draw point at a canvas position and parks the GPUI event loop.
pub use draw_point::draw_point;
/// Returns the first non-demo shape for close-path outcome assertions.
pub use draw_shape::require_draw_shape;
/// Initializes Gauss in the GPUI test application context.
pub use init_app::init_test_app;
/// Performs the initial draw and parks the GPUI event loop before input.
pub use initial_draw::ensure_initial_draw;
