//! GPUI integration tests for file I/O operations.
//!
//! This consolidated test target groups Phase 0 file I/O tests including:
//! - Open dialog and file loading
//! - Save dialog and file writing
//! - Save button interaction
//! - Metadata round-trip preservation

mod common;

mod gpui_file_io {
    mod click_save_button;
    mod metadata_round_trip;
    mod open_dialog;
    mod save_dialog;
}
