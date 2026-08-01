//! Narrow support surface for `gpui_selection_bbox_drag_requires_selection.rs`.

mod add_square;
mod canvas_bounds;
mod init_app;
mod initial_draw;

pub use add_square::add_square;
pub use canvas_bounds::canvas_bounds;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
