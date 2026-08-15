//! Harness facade for the `tests/gpui_history_open_history_reset.rs` integration test.
//!
//! Re-exports app setup, history and selection inspection, and temporary paths
//! for open-file history resets.

#[path = "history.rs"]
mod history;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "read_selection.rs"]
mod read_selection;
#[path = "temp_file_path.rs"]
mod temp_file_path;

/// Provides the harness with the shell's current document-history entry count.
pub use history::read_history_len;
/// Initializes the Gauss application in the harness's GPUI test context.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to become parked.
pub use initial_draw::ensure_initial_draw;
/// Provides the harness with the shell's current selection snapshot.
pub use read_selection::read_selection;
/// Owns a temporary file, exposes its full path, and removes it on drop.
pub use temp_file_path::TempFileGuard;
