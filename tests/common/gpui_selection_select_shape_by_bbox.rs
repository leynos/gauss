//! Narrow support surface for `gpui_selection_select_shape_by_bbox.rs`.

mod canvas_bounds;
mod init_app;
mod initial_draw;

pub use canvas_bounds::canvas_bounds;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
