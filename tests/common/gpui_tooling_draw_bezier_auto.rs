//! Harness facade for the `tests/gpui_tooling_draw_bezier_auto.rs` integration test.
//!
//! Re-exports drawing and canvas setup plus vector assertions for automatic
//! Bézier construction scenarios.

#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "document.rs"]
mod document;
#[path = "draw_point.rs"]
mod draw_point;
#[path = "draw_shape.rs"]
mod draw_shape;
#[path = "escape_input.rs"]
mod escape_input;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "vec2_assertion.rs"]
mod vec2_assertion;

/// Returns the rendered canvas bounds used to position Bézier input points.
pub use canvas_bounds::canvas_bounds;
/// Returns an owned document snapshot for Bézier outcome assertions.
pub use document::read_document;
/// Adds a draw point at a canvas position and parks the GPUI event loop.
pub use draw_point::draw_point;
/// Returns the first non-demo shape for Bézier outcome assertions.
pub use draw_shape::require_draw_shape;
/// Dispatches an unmodified Escape key event and waits for the GPUI event loop to settle.
pub use escape_input::simulate_escape;
/// Initializes Gauss in the GPUI test application context.
pub use init_app::init_test_app;
/// Performs the initial draw and parks the GPUI event loop before input.
pub use initial_draw::ensure_initial_draw;
/// Compares generated Bézier vectors within the harness tolerance.
pub use vec2_assertion::assert_vec2_close;
