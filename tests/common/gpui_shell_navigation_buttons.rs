//! Harness facade for the `tests/gpui_shell_navigation_buttons.rs` integration test.
//!
//! Re-exports drawing, canvas geometry, keyboard escape, and modifier input for
//! navigation-button scenarios.

mod anchor_point;
mod canvas;
mod canvas_bounds;
mod document;
mod draw_point;
mod draw_shape;
mod escape_input;
mod modifiers;

pub use anchor_point::anchor_to_canvas_point;
pub use canvas::CANVAS_PADDING_PX;
pub use canvas_bounds::canvas_bounds;
pub use document::read_document;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use modifiers::shift_secondary;
