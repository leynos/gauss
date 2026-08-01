//! Narrow support surface for `gpui_draw_undo_bdd.rs`.

mod canvas;
mod canvas_bounds;
mod canvas_click;
mod canvas_points;
mod document;
mod document_redo;
mod document_undo;
mod draw_shape;
mod find_draw_shape;
mod init_app;
mod initial_draw;

pub use canvas_click::click_canvas_and_wait;
pub use canvas_points::canvas_points;
pub use document::read_document;
pub use document_redo::simulate_document_redo;
pub use document_undo::simulate_document_undo;
pub use draw_shape::require_draw_shape;
pub use find_draw_shape::find_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
