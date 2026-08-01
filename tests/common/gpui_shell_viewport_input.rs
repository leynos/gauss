//! Narrow support surface for `gpui_shell_viewport_input.rs`.

mod canvas;
mod canvas_bounds;
mod init_app;
mod initial_draw;

pub use canvas::CANVAS_PADDING_PX;
pub use canvas_bounds::canvas_bounds;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
