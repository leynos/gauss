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

/// Provides the harness with an owned snapshot of the shell's current document.
pub use document::read_document;
/// Dispatches document undo and waits for the GPUI event loop to become parked.
pub use document_undo::simulate_document_undo;
/// Provides the harness with the shell's current document-history entry count.
pub use history::read_history_len;
/// Initializes the Gauss application in the harness's GPUI test context.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to become parked.
pub use initial_draw::ensure_initial_draw;
/// Checks that harness-observed vectors are within the helper's tolerance.
pub use vec2_assertion::assert_vec2_close;
