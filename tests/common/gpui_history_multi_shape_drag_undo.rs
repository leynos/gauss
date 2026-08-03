//! Harness facade for the `tests/gpui_history_multi_shape_drag_undo.rs` integration test.
//!
//! Re-exports multi-shape setup, document history controls, and translation
//! assertions for drag undo.

mod add_square;
mod canvas_bounds;
mod document;
mod document_undo;
mod history;
mod init_app;
mod initial_draw;
mod shape_translation;

pub use add_square::add_square;
pub use canvas_bounds::canvas_bounds;
pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use shape_translation::assert_shape_translated_by_delta;
