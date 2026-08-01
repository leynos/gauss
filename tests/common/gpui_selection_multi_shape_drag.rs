//! Narrow support surface for `gpui_selection_multi_shape_drag.rs`.

mod add_square;
mod canvas_bounds;
mod document;
mod init_app;
mod initial_draw;
mod shape_translation;

pub use add_square::add_square;
pub use canvas_bounds::canvas_bounds;
pub use document::read_document;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use shape_translation::assert_shape_translated_by_delta;
