//! Harness facade for the `tests/gpui_tooling_draw_bezier_auto.rs` integration test.
//!
//! Re-exports drawing and canvas setup plus vector assertions for automatic
//! Bézier construction scenarios.

mod canvas_bounds;
mod draw_point;
mod draw_shape;
mod init_app;
mod initial_draw;
mod vec2_assertion;

pub use canvas_bounds::canvas_bounds;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use vec2_assertion::assert_vec2_close;
