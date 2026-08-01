//! Narrow support surface for `gpui_shell_chrome_layout.rs`.

mod canvas_bounds;
mod canvas_click;
mod draw_shape;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use canvas_click::click_canvas_and_wait;
pub use draw_shape::require_draw_shape;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
