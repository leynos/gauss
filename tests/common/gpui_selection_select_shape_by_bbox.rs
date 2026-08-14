//! Harness facade for the `tests/gpui_selection_select_shape_by_bbox.rs` integration test.
//!
//! Re-exports app initialization, initial-draw synchronization, and canvas bounds
//! for bounding-box selection.

mod canvas_bounds;
mod init_app;
mod initial_draw;

/// Returns the rendered canvas bounds used to position the bounding-box click.
pub use canvas_bounds::canvas_bounds;
/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before interaction.
pub use initial_draw::ensure_initial_draw;
