//! Support facade for the `gpui_shell_tool_rail` integration test.
//!
//! It exposes app lifecycle and settled left-click input for exercising the
//! shell tool rail.

mod click_left;
mod init_app;
mod initial_draw;

/// Clicks a tool-rail target and waits for event processing to settle.
pub use click_left::click_left_and_wait;
/// Initializes Gauss for the tool-rail harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
