//! Narrow support surface for `gpui_history_drag_shape_undo.rs`.

mod canvas;
mod canvas_bounds;
mod canvas_drag_delta;
mod canvas_drag_values;
mod document;
mod document_undo;
mod draw_point;
mod draw_shape;
mod history;
mod init_app;
mod initial_draw;
mod manipulate_mode;
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
