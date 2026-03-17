//! GPUI integration tests for window chrome, layout, and shell controls.
//!
//! This consolidated test target groups Phase 0 shell behaviour tests including:
//! - Canvas and chrome layout
//! - Window controls (minimize, maximize, quit)
//! - Tool rail and navigation
//! - Style controls and mode indicators
//! - Viewport input handling
//! - Resize borders and accessibility services

mod common;

mod gpui_shell {
    mod a11y_service;
    mod canvas_layout;
    mod chrome_layout;
    mod mode_indicator;
    mod navigation_buttons;
    mod quit_button;
    mod resize_borders;
    mod style_controls;
    mod tool_rail;
    mod viewport_input;
    mod window_controls;
}
