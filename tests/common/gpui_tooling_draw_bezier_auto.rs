//! Harness facade for the `tests/gpui_tooling_draw_bezier_auto.rs` integration test.
//!
//! Re-exports drawing and canvas setup plus vector assertions for automatic
//! Bézier construction scenarios.

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
#[path = "vec2_assertion.rs"]
mod vec2_assertion;

pub use canvas_bounds::canvas_bounds;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use vec2_assertion::assert_vec2_close;
