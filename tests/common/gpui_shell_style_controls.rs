//! Narrow support surface for `gpui_shell_style_controls.rs`.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod canvas_click;
mod document;
mod document_undo;
mod draw_shape;
mod escape_input;
mod history;
mod init_app;
mod initial_draw;

pub use anchor_point::anchor_to_canvas_point;
pub use canvas_bounds::canvas_bounds;
pub use canvas_click::click_canvas_and_wait;
pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
