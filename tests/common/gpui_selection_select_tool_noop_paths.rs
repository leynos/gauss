//! Harness facade for the `tests/gpui_selection_select_tool_noop_paths.rs` integration test.
//!
//! Re-exports drag scenarios, tool-mode setup, and history and selection
//! inspection for no-op paths.

#[path = "canvas.rs"]
mod canvas;
#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "canvas_drag_points.rs"]
mod canvas_drag_points;
#[path = "canvas_drag_values.rs"]
mod canvas_drag_values;
#[path = "document.rs"]
mod document;
#[path = "draw_point.rs"]
mod draw_point;
#[path = "draw_shape.rs"]
mod draw_shape;
#[path = "history.rs"]
mod history;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "manipulate_mode.rs"]
mod manipulate_mode;
#[path = "read_selection.rs"]
mod read_selection;

/// Returns the rendered canvas bounds used to position no-op interactions.
pub use canvas_bounds::canvas_bounds;
/// Returns a bounded drag scenario for no-op path assertions.
pub use canvas_drag_points::canvas_drag_scenario;
/// Returns an owned document snapshot for no-op state comparisons.
pub use document::read_document;
/// Adds a draw point and parks the GPUI event loop for no-op setup.
pub use draw_point::draw_point;
/// Returns the drawn shape used to verify zero-delta drag behaviour.
pub use draw_shape::require_draw_shape;
/// Returns the current history length for no-op history assertions.
pub use history::read_history_len;
/// Initializes the GPUI application for no-op path scenarios.
pub use init_app::init_test_app;
/// Performs the initial draw and parks GPUI before no-op interactions.
pub use initial_draw::ensure_initial_draw;
/// Switches to manipulate mode and verifies that no shape was created.
pub use manipulate_mode::switch_to_manipulate_mode_and_verify;
/// Returns the shell selection for no-op selection assertions.
pub use read_selection::read_selection;
