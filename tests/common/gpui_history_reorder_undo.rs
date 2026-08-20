//! Harness facade for the `tests/gpui_history_reorder_undo.rs` integration test.
//!
//! Re-exports canvas input, document history and selection inspection, and
//! shape lookup for reorder undo.

mod canvas_bounds;
mod canvas_click;
mod demo_shape;
mod document;
mod document_undo;
mod escape_input;
mod history;
mod init_app;
mod initial_draw;
mod key_input;
mod read_selection;

pub use canvas_bounds::canvas_bounds;
pub use canvas_click::click_canvas_and_wait;
pub use demo_shape::demo_shape_id;
pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use escape_input::simulate_escape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use key_input::simulate_key;
pub use read_selection::read_selection;
