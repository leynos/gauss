//! Harness facade for the `tests/gpui_selection_select_tool_noop_paths.rs` integration test.
//!
//! Re-exports drag scenarios, tool-mode setup, and history and selection
//! inspection for no-op paths.

mod canvas;
mod canvas_bounds;
mod canvas_drag_points;
mod canvas_drag_values;
mod draw_point;
mod history;
mod init_app;
mod initial_draw;
mod manipulate_mode;
mod read_selection;

pub use canvas_bounds::canvas_bounds;
pub use canvas_drag_points::canvas_drag_scenario;
pub use draw_point::draw_point;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use manipulate_mode::switch_to_manipulate_mode_and_verify;
pub use read_selection::read_selection;
