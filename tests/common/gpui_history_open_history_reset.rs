//! Harness facade for the `tests/gpui_history_open_history_reset.rs` integration test.
//!
//! Re-exports app setup, history and selection inspection, and temporary paths
//! for open-file history resets.

mod history;
mod init_app;
mod initial_draw;
mod read_selection;
mod temp_file_path;

pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use read_selection::read_selection;
pub use temp_file_path::TempFileGuard;
