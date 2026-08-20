//! Harness facade for the `tests/gpui_history_close_path_undo.rs` integration test.
//!
//! Re-exports drawing, canvas conversion, document history, and app lifecycle
//! helpers for close-path undo.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod document;
mod document_undo;
mod draw_point;
mod draw_shape;
mod history;
mod init_app;
mod initial_draw;

pub use anchor_point::anchor_to_canvas_point;
pub use canvas_bounds::canvas_bounds;
pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
