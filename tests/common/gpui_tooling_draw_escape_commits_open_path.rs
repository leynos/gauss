//! Harness facade for the `tests/gpui_tooling_draw_escape_commits_open_path.rs` integration test.
//!
//! Re-exports canvas clicks, Escape input, document inspection, and shape lookup
//! for open-path commits.

#[path = "canvas_bounds.rs"]
mod canvas_bounds;
#[path = "canvas_click.rs"]
mod canvas_click;
#[path = "document.rs"]
mod document;
#[path = "draw_shape.rs"]
mod draw_shape;
#[path = "escape_input.rs"]
mod escape_input;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;

/// Looks up the rendered `#phase0-canvas` bounds for harness input.
pub use canvas_bounds::canvas_bounds;
/// Clicks a canvas point without modifiers and waits for the event loop.
pub use canvas_click::click_canvas_and_wait;
/// Provides the harness with an owned snapshot of the shell's current document.
pub use document::read_document;
/// Finds the first drawn shape other than the document's initial demo shape.
pub use draw_shape::require_draw_shape;
/// Dispatches an unmodified Escape key event and waits for the GPUI event loop.
pub use escape_input::simulate_escape;
/// Initializes the Gauss application in the harness's GPUI test context.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to become parked.
pub use initial_draw::ensure_initial_draw;
