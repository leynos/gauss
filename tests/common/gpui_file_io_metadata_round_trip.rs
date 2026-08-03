//! Harness facade for the `tests/gpui_file_io_metadata_round_trip.rs` integration test.
//!
//! Re-exports app setup, initial-draw synchronization, and guarded temporary
//! files for metadata round trips.

mod init_app;
mod initial_draw;
mod temp_file;

pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use temp_file::TempFileGuard;
