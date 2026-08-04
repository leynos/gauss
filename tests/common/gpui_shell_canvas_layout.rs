//! Harness facade for the `tests/gpui_shell_canvas_layout.rs` integration test.
//!
//! Re-exports app setup and canvas bounds for shell layout assertions.

mod canvas_bounds;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
