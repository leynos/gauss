//! Harness facade for the `tests/gpui_selection_select_tool_noop_paths.rs` integration test.
//!
//! Re-exports drag scenarios, tool-mode setup, and history and selection
//! inspection for no-op paths.

mod canvas;
mod canvas_bounds;
mod canvas_drag_points;
mod canvas_drag_values;
mod document;
mod draw_point;
mod draw_shape;
mod history;
mod init_app;
mod initial_draw;
mod manipulate_mode;
mod read_selection;

pub use canvas_bounds::canvas_bounds;
pub use canvas_drag_points::canvas_drag_scenario;
/// Returns an owned document snapshot for no-op state comparisons.
pub use document::read_document;
pub use draw_point::draw_point;
/// Returns the drawn shape used to verify zero-delta drag behaviour.
pub use draw_shape::require_draw_shape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use manipulate_mode::switch_to_manipulate_mode_and_verify;
pub use read_selection::read_selection;
