//! Harness facade for the `tests/gpui_history_command_grouping_undo.rs` integration test.
//!
//! Re-exports app setup, document history controls, history inspection, and
//! vector assertions for grouped undo.

mod document;
mod document_undo;
mod history;
mod init_app;
mod initial_draw;
mod vec2_assertion;

pub use document::read_document;
pub use document_undo::simulate_document_undo;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use vec2_assertion::assert_vec2_close;
