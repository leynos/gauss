//! Support facade for the `gpui_file_io_open_dialog` integration test.
//!
//! It exposes app lifecycle and guarded temporary-path capabilities required
//! to exercise the open-dialog harness.

#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "temp_file.rs"]
mod temp_file;

/// Initializes Gauss for the open-dialog harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
/// Guards the full temporary path used by the open-dialog lifecycle.
pub use temp_file::TempFileGuard;
