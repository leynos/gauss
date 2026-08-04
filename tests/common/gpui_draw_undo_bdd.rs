//! Harness facade for the `tests/gpui_draw_undo_bdd.rs` integration test.
//!
//! Re-exports app and canvas setup, document history controls, and shape lookup
//! helpers for draw undo scenarios.

mod canvas;
mod canvas_bounds;
mod canvas_click;
mod canvas_points;
mod document;
mod document_redo;
mod document_undo;
mod draw_shape;
mod find_draw_shape;
mod init_app;
mod initial_draw;

/// Moves to a canvas point, clicks, and returns after the event loop parks.
pub use canvas_click::click_canvas_and_wait;
/// Returns opposing padded canvas points or an error when bounds are missing.
pub use canvas_points::canvas_points;
/// Returns an owned snapshot of the shell's current document.
pub use document::read_document;
/// Dispatches document redo and returns after the event loop parks.
pub use document_redo::simulate_document_redo;
/// Dispatches document undo and returns after the event loop parks.
pub use document_undo::simulate_document_undo;
/// Borrows the first non-demo shape or returns a missing-shape error.
pub use draw_shape::require_draw_shape;
/// Borrows the first non-demo shape, returning `None` when none exists.
pub use find_draw_shape::find_draw_shape;
/// Initializes Gauss in the supplied GPUI test application context.
pub use init_app::init_test_app;
/// Performs the initial draw and returns after the event loop parks.
pub use initial_draw::ensure_initial_draw;
