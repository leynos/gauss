//! Harness facade for the `tests/gpui_tooling_draw_escape_commits_open_path.rs` integration test.
//!
//! Re-exports canvas clicks, escape input, document inspection, and shape lookup
//! for open-path commits.

mod canvas_bounds;
mod canvas_click;
mod document;
mod draw_shape;
mod escape_input;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use canvas_click::click_canvas_and_wait;
pub use document::read_document;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
