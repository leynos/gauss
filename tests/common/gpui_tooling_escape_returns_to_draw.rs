//! Narrow support surface for `gpui_tooling_escape_returns_to_draw.rs`.

mod canvas;
mod canvas_bounds;
mod canvas_drag_delta;
mod canvas_drag_values;
mod document;
mod draw_point;
mod draw_shape;
mod escape_input;
mod history;
mod init_app;
mod initial_draw;
mod manipulate_mode;

pub use canvas_bounds::canvas_bounds;
pub use canvas_drag_delta::CanvasDragScenario;
pub use canvas_drag_delta::canvas_drag_scenario;
pub use document::read_document;
pub use draw_point::draw_point;
pub use draw_shape::require_draw_shape;
pub use escape_input::simulate_escape;
pub use history::read_history_len;
pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
pub use manipulate_mode::switch_to_manipulate_mode_and_verify;
