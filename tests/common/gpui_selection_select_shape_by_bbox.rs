//! Harness facade for the `tests/gpui_selection_select_shape_by_bbox.rs` integration test.
//!
//! Re-exports app initialization, initial-draw synchronization, and canvas bounds
//! for bounding-box selection.

mod canvas_bounds;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
