//! Accessibility preparation for future AccessKit integration.
//!
//! This module defines stable node IDs and semantic metadata for window
//! controls, preparing for full AccessKit integration as outlined in
//! `docs/accesskit-based-accessibility-in-gpui.md`.
//!
//! When AccessKit is integrated, each window control will expose:
//!
//! - **Role**: Button, `ToggleButton`, etc.
//! - **Name**: Descriptive label announced by screen readers
//! - **Actions**: Click/Invoke mapped to GPUI actions
//! - **States**: Pressed/checked for toggle buttons (maximize, fullscreen)
//!
//! The node IDs defined here are deterministic and stable across sessions,
//! which is a requirement for AccessKit's immediate-mode bridge.
//!
//! Note: Constants in this module are intentionally defined for future use
//! and may appear unused until AccessKit integration is complete.

/// Stable node IDs for window control elements.
///
/// These IDs are used to identify accessibility nodes for window controls.
/// AccessKit requires stable IDs that persist across frames so assistive
/// technologies can track focus and state changes.
///
/// The ID scheme uses a reserved range (0x1000–0x1FFF) for window chrome
/// elements, leaving other ranges for document content.
#[expect(
    dead_code,
    reason = "Constants prepared for future AccessKit integration"
)]
pub mod node_ids {
    /// Minimize window button.
    pub const MINIMIZE_BUTTON: u64 = 0x1001;
    /// Maximize/restore window button.
    pub const MAXIMIZE_BUTTON: u64 = 0x1002;
    /// Close window button.
    pub const CLOSE_BUTTON: u64 = 0x1003;
    /// Toggle fullscreen button (if exposed separately).
    pub const FULLSCREEN_BUTTON: u64 = 0x1004;
    /// System window menu (Alt+Space).
    pub const WINDOW_MENU: u64 = 0x1005;
    /// Titlebar drag region.
    pub const TITLEBAR: u64 = 0x1006;
}

/// Accessible names for window control buttons.
///
/// These strings are announced by screen readers when focus moves to
/// each control. They include keyboard shortcuts for discoverability.
#[expect(
    dead_code,
    reason = "Constants prepared for future AccessKit integration"
)]
pub mod accessible_names {
    /// Accessible name for the minimize button.
    pub const MINIMIZE: &str = "Minimize window";
    /// Accessible name for the maximize button (when not maximized).
    pub const MAXIMIZE: &str = "Maximize window";
    /// Accessible name for the restore button (when maximized).
    pub const RESTORE: &str = "Restore window";
    /// Accessible name for the close button.
    pub const CLOSE: &str = "Close window";
    /// Accessible name for the fullscreen toggle.
    pub const FULLSCREEN: &str = "Toggle fullscreen";
    /// Accessible name for the window menu.
    pub const WINDOW_MENU: &str = "Window menu";
}

/// Keyboard shortcut hints for accessibility announcements.
///
/// Screen readers can announce these along with button names to help
/// users learn keyboard shortcuts.
#[expect(
    dead_code,
    reason = "Constants prepared for future AccessKit integration"
)]
pub mod shortcut_hints {
    /// Shortcut hint for minimize (cross-platform).
    pub const MINIMIZE: &str = "Alt+F9";
    /// Shortcut hint for maximize (cross-platform).
    pub const MAXIMIZE: &str = "Alt+F10";
    /// Shortcut hint for close (cross-platform).
    pub const CLOSE: &str = "Alt+F4";
    /// Shortcut hint for fullscreen (cross-platform).
    pub const FULLSCREEN: &str = "Alt+F11";
    /// Shortcut hint for window menu (cross-platform).
    pub const WINDOW_MENU: &str = "Alt+Space";

    /// Shortcut hint for minimize on macOS.
    #[cfg(target_os = "macos")]
    pub const MINIMIZE_MACOS: &str = "Cmd+M";
    /// Shortcut hint for close on macOS.
    #[cfg(target_os = "macos")]
    pub const CLOSE_MACOS: &str = "Cmd+Q";
    /// Shortcut hint for fullscreen on macOS.
    #[cfg(target_os = "macos")]
    pub const FULLSCREEN_MACOS: &str = "Ctrl+Cmd+F";
}
