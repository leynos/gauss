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
//! The constants in this module are consumed by `a11y_service` for 0.6.1 and
//! are intended to remain stable as later accessibility milestones are
//! implemented.

use std::collections::BTreeMap;

use accesskit::{Node, NodeId, TreeUpdate};

/// Canonical semantics for a chrome button in the accessibility tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChromeButtonSemantics {
    /// Stable node ID used in AccessKit updates.
    pub node_id: u64,
    /// Accessible label announced by assistive technologies.
    pub label: &'static str,
    /// Keyboard hint announced alongside the label.
    pub shortcut_hint: &'static str,
}

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
    /// Accessible name for the titlebar region.
    pub const TITLEBAR: &str = "Window title bar";
}

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

    /// Return the minimize shortcut hint for the current platform.
    #[cfg(target_os = "macos")]
    #[must_use]
    pub const fn minimize_for_platform() -> &'static str {
        MINIMIZE_MACOS
    }

    /// Return the minimize shortcut hint for the current platform.
    #[cfg(not(target_os = "macos"))]
    #[must_use]
    pub const fn minimize_for_platform() -> &'static str {
        MINIMIZE
    }

    /// Return the close shortcut hint for the current platform.
    #[cfg(target_os = "macos")]
    #[must_use]
    pub const fn close_for_platform() -> &'static str {
        CLOSE_MACOS
    }

    /// Return the close shortcut hint for the current platform.
    #[cfg(not(target_os = "macos"))]
    #[must_use]
    pub const fn close_for_platform() -> &'static str {
        CLOSE
    }

    /// Return the fullscreen shortcut hint for the current platform.
    #[cfg(target_os = "macos")]
    #[must_use]
    pub const fn fullscreen_for_platform() -> &'static str {
        FULLSCREEN_MACOS
    }

    /// Return the fullscreen shortcut hint for the current platform.
    #[cfg(not(target_os = "macos"))]
    #[must_use]
    pub const fn fullscreen_for_platform() -> &'static str {
        FULLSCREEN
    }
}

/// Return canonical semantics for chrome button nodes in deterministic order.
#[must_use]
pub const fn chrome_button_semantics(is_maximized: bool) -> [ChromeButtonSemantics; 5] {
    [
        ChromeButtonSemantics {
            node_id: node_ids::WINDOW_MENU,
            label: accessible_names::WINDOW_MENU,
            shortcut_hint: shortcut_hints::WINDOW_MENU,
        },
        ChromeButtonSemantics {
            node_id: node_ids::MINIMIZE_BUTTON,
            label: accessible_names::MINIMIZE,
            shortcut_hint: shortcut_hints::minimize_for_platform(),
        },
        ChromeButtonSemantics {
            node_id: node_ids::MAXIMIZE_BUTTON,
            label: if is_maximized {
                accessible_names::RESTORE
            } else {
                accessible_names::MAXIMIZE
            },
            shortcut_hint: shortcut_hints::MAXIMIZE,
        },
        ChromeButtonSemantics {
            node_id: node_ids::FULLSCREEN_BUTTON,
            label: accessible_names::FULLSCREEN,
            shortcut_hint: shortcut_hints::fullscreen_for_platform(),
        },
        ChromeButtonSemantics {
            node_id: node_ids::CLOSE_BUTTON,
            label: accessible_names::CLOSE,
            shortcut_hint: shortcut_hints::close_for_platform(),
        },
    ]
}

/// Return a chrome node from a node map keyed by AccessKit IDs.
#[must_use]
pub fn chrome_node_from_map(nodes: &BTreeMap<NodeId, Node>, node_id: u64) -> Option<&Node> {
    nodes.get(&NodeId(node_id))
}

/// Return a chrome node from a serialized `TreeUpdate` payload.
#[must_use]
pub fn chrome_node_from_update(update: &TreeUpdate, node_id: u64) -> Option<&Node> {
    update
        .nodes
        .iter()
        .find_map(|(candidate, node)| (candidate.0 == node_id).then_some(node))
}
