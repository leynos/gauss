//! Harness facade for the `tests/gpui_selection_bbox_drag_requires_selection.rs` integration test.
//!
//! Re-exports app and canvas setup plus square creation for selection-required bounding-box drags.

mod add_square;
mod canvas_bounds;
mod init_app;
mod initial_draw;

/// Appends a closed square for bounding-box drag setup and returns its shape ID.
pub use add_square::add_square;
/// Returns the rendered canvas bounds used to map bounding-box input points.
pub use canvas_bounds::canvas_bounds;
/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before interaction.
pub use initial_draw::ensure_initial_draw;
