//! Narrow support surface for `gpui_shell_a11y_service.rs`.

mod add_square;
mod init_app;
mod initial_draw;

pub use add_square::add_square;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
