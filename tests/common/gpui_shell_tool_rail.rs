//! Narrow support surface for `gpui_shell_tool_rail.rs`.

mod click_left;
mod init_app;
mod initial_draw;

pub use click_left::click_left_and_wait;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
