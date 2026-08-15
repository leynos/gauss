//! Harness facade for the `tests/gpui_selection_clear_selection.rs` integration test.
//!
//! Re-exports app and canvas setup, pointer input, and selection snapshots for
//! clear-selection scenarios.

mod canvas_bounds;
mod click_left;
mod init_app;
mod initial_draw;
mod read_selection;

/// Returns the rendered canvas bounds used to choose an empty click position.
pub use canvas_bounds::canvas_bounds;
/// Simulates an unmodified left click and parks the event loop afterwards.
pub use click_left::click_left_and_wait;
/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before interaction.
pub use initial_draw::ensure_initial_draw;
/// Returns the shell view's owned selection snapshot after the canvas click.
pub use read_selection::read_selection;
