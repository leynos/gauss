//! Support facade for the `gpui_shell_a11y_service` integration test.
//!
//! It exposes square-document setup and rendering capabilities used to inspect
//! the shell accessibility service.

mod add_square;
mod initial_draw;

/// Appends the square fixture inspected through the accessibility tree.
pub use add_square::add_square;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
