//! Harness facade for the `tests/gpui_shell_chrome_layout.rs` integration test.
//!
//! Re-exports the lifecycle helpers required by the retained layout checks.

mod init_app;
mod initial_draw;

/// Initializes the Gauss application inside a GPUI test context.
pub use init_app::init_test_app;
/// Performs the initial window draw and drains the event loop until parked.
pub use initial_draw::ensure_initial_draw;
