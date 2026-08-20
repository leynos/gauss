//! Harness facade for the `tests/gpui_shell_viewport_input.rs` integration test.
//!
//! Re-exports app setup, canvas bounds, and the canvas inset for viewport input assertions.

mod canvas;
mod canvas_bounds;
mod init_app;
mod initial_draw;

/// Provides the inset used to keep viewport input points inside the canvas.
pub use canvas::CANVAS_PADDING_PX;
/// Returns the rendered canvas bounds used to position viewport input.
pub use canvas_bounds::canvas_bounds;
/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before input.
pub use initial_draw::ensure_initial_draw;
