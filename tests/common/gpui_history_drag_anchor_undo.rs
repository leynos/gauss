//! Narrow support surface for `gpui_history_drag_anchor_undo.rs`.

mod canvas;
mod canvas_bounds;
mod canvas_drag_anchor;
mod canvas_drag_values;
mod document;
mod document_undo;
mod draw_point;
mod draw_shape;
mod escape_input;
mod history;
mod init_app;
mod initial_draw;
mod vec2_assertion;

pub use canvas_drag_anchor::CanvasDragScenario;
pub use canvas_drag_anchor::canvas_drag_scenario;
pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use vec2_assertion::assert_vec2_close;
