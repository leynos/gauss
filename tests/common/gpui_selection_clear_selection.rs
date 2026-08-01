//! Narrow support surface for `gpui_selection_clear_selection.rs`.

mod canvas_bounds;
mod click_left;
mod init_app;
mod initial_draw;
mod selection;

pub use canvas_bounds::canvas_bounds;
pub use click_left::click_left_and_wait;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use selection::read_selection_items;
