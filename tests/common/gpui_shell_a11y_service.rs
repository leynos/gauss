//! Support facade for the `gpui_shell_a11y_service` integration test.
//!
//! It exposes square-document setup used to inspect the shell accessibility
//! service.

mod add_square;

/// Appends the square fixture inspected through the accessibility tree.
pub use add_square::add_square;
