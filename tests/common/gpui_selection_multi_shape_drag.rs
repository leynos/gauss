//! Harness facade for the `tests/gpui_selection_multi_shape_drag.rs` integration test.
//!
//! Re-exports multi-shape and canvas setup, document inspection, and translation
//! assertions for selection drags.

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

/// Appends a closed square for multi-shape drag setup and returns its shape ID.
pub use add_square::add_square;
/// Returns the rendered canvas bounds used to map drag input points.
pub use canvas_bounds::canvas_bounds;
/// Copies the shell view's current document for before-and-after comparisons.
pub use document::read_document;
/// Initializes the Gauss application in the GPUI test context.
pub use init_app::init_test_app;
/// Performs the first window draw and parks the event loop before interaction.
pub use initial_draw::ensure_initial_draw;
/// Checks that every anchor moved by the expected document-space delta.
pub use shape_translation::assert_shape_translated_by_delta;
