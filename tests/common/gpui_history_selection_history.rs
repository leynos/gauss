//! Harness facade for the `tests/gpui_history_selection_history.rs` integration test.
//!
//! Re-exports drawing and pointer input, app setup, and selection history
//! controls for selection scenarios.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod click_left;
mod draw_point;
mod draw_shape;
mod escape_input;
mod init_app;
mod initial_draw;
mod selection_redo;
mod selection_undo;

pub use anchor_point::anchor_to_canvas_point;
pub use canvas_bounds::canvas_bounds;
pub use click_left::click_left_and_wait;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use selection_redo::simulate_selection_redo;
pub use selection_undo::simulate_selection_undo;
