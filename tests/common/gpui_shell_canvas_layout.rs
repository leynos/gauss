//! Harness facade for the `tests/gpui_shell_canvas_layout.rs` integration test.
//!
//! Re-exports app setup, canvas bounds, and the minimum canvas height for shell layout assertions.

mod canvas_bounds;
mod canvas_height;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use canvas_height::MIN_CANVAS_HEIGHT_PX;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
