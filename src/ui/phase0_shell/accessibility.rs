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

#[expect(
    dead_code,
    reason = "Constants prepared for future AccessKit integration"
)]
pub mod node_ids {
    //! Stable node IDs for window control accessibility nodes.
    //!
    //! These constants assign deterministic identifiers to window chrome elements,
    //! enabling AccessKit to track focus and state changes across frames.

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

#[expect(
    dead_code,
    reason = "Constants prepared for future AccessKit integration"
)]
pub mod accessible_names {
    //! Human-readable names announced by screen readers.
    //!
    //! These strings provide descriptive labels for assistive technologies,
    //! enabling users to identify window controls by their spoken names.

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

#[expect(
    dead_code,
    reason = "Constants prepared for future AccessKit integration"
)]
pub mod shortcut_hints {
    //! Keyboard shortcut hints for accessibility announcements.
    //!
    //! Screen readers announce these alongside button names to help users
    //! discover and remember keyboard shortcuts for common operations.

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
