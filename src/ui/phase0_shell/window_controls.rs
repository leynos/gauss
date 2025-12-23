//! Keyboard-accessible window control operations.
//!
//! This module provides keyboard-driven window management (move, resize,
//! minimize, maximize, close, fullscreen) with a focus on accessibility.
//! The design prepares for future AccessKit integration by:
//!
//! - Defining clear action handlers that can be exposed as AccessKit actions
//! - Maintaining stable operation semantics across platforms
//! - Providing state queries (`is_maximized`, `is_fullscreen`) for AccessKit
//!   nodes
//!
//! Note: State query functions are prepared for future AccessKit integration
//! and may appear unused until that work is complete.

use gpui::{ResizeEdge, Window, point, px};

/// Minimize the window.
pub fn minimize(window: &Window) {
    window.minimize_window();
}

/// Toggle maximize/restore state.
pub fn toggle_maximize(window: &Window) {
    window.zoom_window();
}

/// Toggle fullscreen mode.
pub fn toggle_fullscreen(window: &Window) {
    window.toggle_fullscreen();
}

/// Check if the window is currently maximized.
#[expect(
    dead_code,
    reason = "State query prepared for future AccessKit integration"
)]
#[must_use]
pub fn is_maximized(window: &Window) -> bool {
    window.is_maximized()
}

/// Check if the window is currently in fullscreen mode.
#[expect(
    dead_code,
    reason = "State query prepared for future AccessKit integration"
)]
#[must_use]
pub fn is_fullscreen(window: &Window) -> bool {
    window.is_fullscreen()
}

/// Start compositor-controlled window move.
///
/// On Linux (Wayland/X11), this delegates to the compositor which handles
/// the interactive move operation. On macOS, the compositor may ignore
/// this call if native titlebar dragging is active; the behaviour depends
/// on the GPUI backend implementation.
pub fn start_move(window: &Window) {
    window.start_window_move();
}

/// Start compositor-controlled window resize from the specified edge.
pub fn start_resize(window: &Window, edge: ResizeEdge) {
    window.start_window_resize(edge);
}

/// Show the system window menu at a default location near the top-left.
pub fn show_window_menu(window: &Window) {
    window.show_window_menu(point(px(10.0), px(30.0)));
}
