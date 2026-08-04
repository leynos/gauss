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

pub use anchor_point::anchor_to_canvas_point;
pub use canvas_bounds::canvas_bounds;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
