//! Harness facade for the `tests/gpui_tooling_escape_returns_to_draw.rs` integration test.
//!
//! Re-exports drag and drawing setup, tool-mode transitions, and history
//! inspection for escape handling.

// Required transitively by `canvas_drag_values` through `super::canvas`.
mod canvas;
mod canvas_bounds;
mod canvas_click;
mod canvas_drag_delta;
// Required transitively by `canvas_drag_delta` through `super::canvas_drag_values`.
mod canvas_drag_values;
mod document;
mod draw_point;
mod draw_shape;
mod escape_input;
mod history;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use canvas_click::click_canvas_and_wait;
pub use canvas_drag_delta::canvas_drag_scenario;
pub use document::read_document;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
