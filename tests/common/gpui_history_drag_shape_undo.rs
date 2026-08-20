//! Harness facade for the `tests/gpui_history_drag_shape_undo.rs` integration test.
//!
//! Re-exports shape-drag setup, document history controls, and translation
//! assertions for shape-drag undo.

#[path = "canvas.rs"]
mod canvas;
#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "canvas_drag_delta.rs"]
mod canvas_drag_delta;
#[path = "canvas_drag_values.rs"]
mod canvas_drag_values;
#[path = "document.rs"]
mod document;
#[path = "document_undo.rs"]
mod document_undo;
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
#[path = "shape_translation.rs"]
mod shape_translation;

pub use canvas_drag_delta::canvas_drag_scenario;
pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use manipulate_mode::switch_to_manipulate_mode_and_verify;
pub use shape_translation::assert_shape_translated_by_delta;
