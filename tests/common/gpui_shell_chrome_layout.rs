//! Harness facade for the `tests/gpui_shell_chrome_layout.rs` integration test.
//!
//! Re-exports the lifecycle helpers required by the retained layout checks.

mod init_app;
mod initial_draw;

pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
