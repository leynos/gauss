//! Harness facade for the `tests/gpui_file_io_metadata_round_trip.rs` integration test.
//!
//! Re-exports app setup, initial-draw synchronization, and guarded temporary
//! files for metadata round trips.

#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "temp_file.rs"]
mod temp_file;

/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before metadata I/O.
pub use initial_draw::ensure_initial_draw;
/// Owns a temporary metadata file and removes it when the guard is dropped.
pub use temp_file::TempFileGuard;
