//! Narrow support surface for `gpui_file_io_save_dialog.rs`.

mod init_app;
mod initial_draw;
mod temp_file_save;

pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use temp_file_save::TempFileGuard;
