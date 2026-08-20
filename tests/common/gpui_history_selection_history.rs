//! Harness facade for the `tests/gpui_history_selection_history.rs` integration test.
//!
//! Re-exports drawing and pointer input plus app setup for selection scenarios.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod click_left;
mod draw_point;
mod draw_shape;
mod escape_input;
mod init_app;
mod initial_draw;

pub use anchor_point::anchor_to_canvas_point;
pub use canvas_bounds::canvas_bounds;
pub use click_left::click_left_and_wait;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
