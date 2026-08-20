//! Support facade for the `gpui_shell_a11y_service` integration test.
//!
//! It exposes app lifecycle and square-document setup capabilities used to
//! inspect the shell accessibility service.

mod add_square;
mod init_app;
mod initial_draw;

/// Appends the square fixture inspected through the accessibility tree.
pub use add_square::add_square;
/// Initializes Gauss for the accessibility-service harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
