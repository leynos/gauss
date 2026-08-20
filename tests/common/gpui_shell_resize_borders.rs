//! Harness facade for the `tests/gpui_shell_resize_borders.rs` integration test.
//!
//! Re-exports app initialization and initial-draw synchronization for shell
//! resize-border assertions.

mod init_app;
mod initial_draw;

pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
