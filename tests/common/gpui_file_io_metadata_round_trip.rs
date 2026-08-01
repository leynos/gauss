//! Narrow support surface for `gpui_file_io_metadata_round_trip.rs`.

mod init_app;
mod initial_draw;
mod temp_file;

pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use temp_file::TempFileGuard;
