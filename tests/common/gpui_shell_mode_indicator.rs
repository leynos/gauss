//! Support facade for the `gpui_shell_mode_indicator` integration test.
//!
//! It exposes the app lifecycle capabilities needed to render and inspect the
//! shell mode indicator.

mod init_app;
mod initial_draw;

/// Initializes Gauss for the mode-indicator harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
