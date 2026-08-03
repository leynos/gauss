//! Support facade for the `gpui_file_io_save_dialog` integration test.
//!
//! It exposes app lifecycle and temporary save-target capabilities required by
//! the save-dialog harness.

mod init_app;
mod initial_draw;
mod temp_file_save;

/// Initializes Gauss for the save-dialog harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
/// Guards the temporary target used throughout the save-dialog lifecycle.
pub use temp_file_save::TempFileGuard;
