//! Harness facade for the `tests/gpui_history_draw_undo.rs` integration test.
//!
//! Re-exports app and canvas setup, click interaction, and document inspection
//! for drawing undo scenarios.

mod canvas_bounds;
mod canvas_click;
mod document;
mod draw_shape;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use canvas_click::click_canvas_and_wait;
pub use document::read_document;
pub use draw_shape::require_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
