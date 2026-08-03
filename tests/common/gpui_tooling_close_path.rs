//! Harness facade for the `tests/gpui_tooling_close_path.rs` integration test.
//!
//! Re-exports app and canvas setup, drawing input, and shape inspection for
//! close-path tooling scenarios.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod draw_point;
mod draw_shape;
mod init_app;
mod initial_draw;

pub use anchor_point::anchor_to_canvas_point;
pub use canvas_bounds::canvas_bounds;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
