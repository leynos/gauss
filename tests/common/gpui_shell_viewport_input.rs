//! Harness facade for the `tests/gpui_shell_viewport_input.rs` integration test.
//!
//! Re-exports canvas bounds for viewport input assertions.

mod canvas_bounds;

/// Returns the rendered canvas bounds used to position viewport input.
pub use canvas_bounds::canvas_bounds;
