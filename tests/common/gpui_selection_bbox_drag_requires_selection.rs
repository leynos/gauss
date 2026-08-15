//! Harness facade for the `tests/gpui_selection_bbox_drag_requires_selection.rs` integration test.
//!
//! Re-exports app and canvas setup plus document assertions for selection-
//! required bounding-box drags.

#[path = "add_square.rs"]
mod add_square;
#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "document.rs"]
mod document;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "shape_translation.rs"]
mod shape_translation;

/// Appends a closed square for bounding-box drag setup and returns its shape ID.
pub use add_square::add_square;
/// Returns the rendered canvas bounds used to map bounding-box input points.
pub use canvas_bounds::canvas_bounds;
/// Returns an owned document snapshot for post-drag assertions.
pub use document::read_document;
/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before interaction.
pub use initial_draw::ensure_initial_draw;
/// Verifies that the bounding-box drag changed anchors by the expected delta.
pub use shape_translation::assert_shape_translated_by_delta;
