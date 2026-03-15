//! Platform-independent keystroke representation.
//!
//! This module provides types for representing keyboard shortcuts without
//! depending on a UI framework, enabling testing and serialization of
//! keybindings.
//!
//! # Design
//!
//! The [`Keystroke`] type captures the key and modifiers for a keyboard
//! shortcut. It provides:
//!
//! - Builder methods for ergonomic construction
//! - Human-readable display for UI and documentation
//! - Stable modifier ordering for external serializers
//!
//! # Platform Modifiers
//!
//! The [`Modifiers`] type uses a `secondary` flag for the platform-specific
//! "command" modifier (Cmd on macOS, Ctrl on other platforms), keeping
//! keystrokes consistent across platforms while remaining framework-agnostic.
//!
//! # Examples
//!
//! ```rust,no_run
//! use gauss_core::model::Keystroke;
//!
//! // Simple key
//! let tab = Keystroke::new("tab");
//! assert_eq!(tab.key, "tab");
//!
//! // Platform "secondary" modifier (Cmd on macOS, Ctrl elsewhere)
//! let undo = Keystroke::secondary("z");
//! assert!(undo.modifiers.secondary);
//!
//! // Multiple modifiers
//! let redo = Keystroke::secondary_shift("z");
//! ```

#[cfg(test)]
mod tests;

use std::fmt;

/// Individual modifier key type.
///
/// Used by [`Modifiers::active_in_order`] to provide a single source of truth
/// for modifier ordering and iteration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    /// Control key (Ctrl).
    Control,
    /// Alt/Option key.
    Alt,
    /// Shift key.
    Shift,
    /// Platform "secondary" modifier: Cmd on macOS, Ctrl elsewhere.
    Secondary,
}

/// Modifier keys for a keystroke.
///
/// Modifiers track which modifier keys are held during a keystroke. The
/// `secondary` field represents the platform-specific "command" modifier
/// (Cmd on macOS, Ctrl on other platforms).
///
/// # Examples
///
/// ```rust,no_run
/// use gauss_core::model::Modifiers;
///
/// let mods = Modifiers::default().with_secondary().with_shift();
/// assert!(mods.secondary);
/// assert!(mods.shift);
/// assert!(!mods.alt);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Keyboard modifiers are naturally represented as bools"
)]
pub struct Modifiers {
    /// Control key (Ctrl).
    pub control: bool,
    /// Alt/Option key.
    pub alt: bool,
    /// Shift key.
    pub shift: bool,
    /// Platform "secondary" modifier: Cmd on macOS, Ctrl elsewhere.
    ///
    /// When this is true, the keystroke uses the platform's standard
    /// "command" modifier (Cmd on macOS, Ctrl on other platforms).
    pub secondary: bool,
}

impl Modifiers {
    /// Return modifiers with the secondary (Cmd/Ctrl) flag set.
    #[must_use]
    pub const fn with_secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    /// Return modifiers with the shift flag set.
    #[must_use]
    pub const fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// Return modifiers with the alt flag set.
    #[must_use]
    pub const fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Return modifiers with the control flag set.
    #[must_use]
    pub const fn with_control(mut self) -> Self {
        self.control = true;
        self
    }

    /// Check if any modifier is set.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.control || self.alt || self.shift || self.secondary
    }

    /// Check if no modifiers are set.
    #[must_use]
    pub const fn none(&self) -> bool {
        !self.any()
    }

    /// Return an iterator of active modifiers in canonical order.
    ///
    /// The order matches the canonical modifier ordering used by UI
    /// formatters: Control, Alt, Shift, Secondary (Cmd/Ctrl).
    ///
    /// This centralizes modifier ordering logic so all output methods
    /// (serializers, display names) share a single source of truth.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::{Modifiers, Modifier};
    ///
    /// let mods = Modifiers::default().with_secondary().with_shift();
    /// let active: Vec<_> = mods.active_in_order().collect();
    /// assert_eq!(active, vec![Modifier::Shift, Modifier::Secondary]);
    /// ```
    pub fn active_in_order(&self) -> impl Iterator<Item = Modifier> + '_ {
        [
            (self.control, Modifier::Control),
            (self.alt, Modifier::Alt),
            (self.shift, Modifier::Shift),
            (self.secondary, Modifier::Secondary),
        ]
        .into_iter()
        .filter_map(|(enabled, m)| enabled.then_some(m))
    }
}

/// A platform-independent keystroke representation.
///
/// This type captures keyboard shortcuts without UI framework dependency,
/// enabling testing and serialization of keybindings.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss_core::model::Keystroke;
///
/// // Simple key
/// let delete = Keystroke::new("backspace");
///
/// // With modifiers
/// let select_all = Keystroke::secondary("a");
/// let deselect = Keystroke::secondary_shift("a");
///
/// assert!(select_all.modifiers.secondary);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Keystroke {
    /// The key or character (e.g., "z", "tab", "backspace", "f4").
    pub key: String,
    /// Modifier keys held during the keystroke.
    pub modifiers: Modifiers,
}

impl Keystroke {
    /// Create a keystroke with no modifiers.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::Keystroke;
    ///
    /// let tab = Keystroke::new("tab");
    /// assert_eq!(tab.key, "tab");
    /// ```
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::default(),
        }
    }

    /// Create a keystroke with the secondary (Cmd/Ctrl) modifier.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::Keystroke;
    ///
    /// let undo = Keystroke::secondary("z");
    /// assert!(undo.modifiers.secondary);
    /// ```
    #[must_use]
    pub fn secondary(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::default().with_secondary(),
        }
    }

    /// Create a keystroke with secondary + shift modifiers.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::Keystroke;
    ///
    /// let redo = Keystroke::secondary_shift("z");
    /// assert!(redo.modifiers.secondary);
    /// assert!(redo.modifiers.shift);
    /// ```
    #[must_use]
    pub fn secondary_shift(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::default().with_secondary().with_shift(),
        }
    }

    /// Create a keystroke with the alt modifier.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::Keystroke;
    ///
    /// let close = Keystroke::alt("f4");
    /// assert!(close.modifiers.alt);
    /// ```
    #[must_use]
    pub fn alt(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::default().with_alt(),
        }
    }

    /// Create a keystroke with custom modifiers.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::{Keystroke, Modifiers};
    ///
    /// let mods = Modifiers::default().with_control().with_shift();
    /// let keystroke = Keystroke::with_modifiers("a", mods);
    /// assert!(keystroke.modifiers.control);
    /// assert!(keystroke.modifiers.shift);
    /// ```
    #[must_use]
    pub fn with_modifiers(key: impl Into<String>, modifiers: Modifiers) -> Self {
        Self {
            key: key.into(),
            modifiers,
        }
    }

    /// Return the human-readable display name for this keystroke.
    ///
    /// This produces platform-aware output suitable for UI display:
    ///
    /// - macOS: Uses symbols (⌘, ⌥, ⇧, ⌃)
    /// - Other platforms: Uses text (Ctrl, Alt, Shift)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::Keystroke;
    ///
    /// let undo = Keystroke::secondary("z");
    /// // macOS: "⌘Z", Linux/Windows: "Ctrl+Z"
    /// ```
    #[must_use]
    pub fn display_name(&self) -> String {
        #[cfg(target_os = "macos")]
        {
            self.display_name_macos()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.display_name_other()
        }
    }

    #[cfg(target_os = "macos")]
    fn display_name_macos(&self) -> String {
        let mut out = String::new();

        for m in self.modifiers.active_in_order() {
            out.push_str(match m {
                Modifier::Control => "⌃",
                Modifier::Alt => "⌥",
                Modifier::Shift => "⇧",
                Modifier::Secondary => "⌘",
            });
        }

        out.push_str(&self.key.to_uppercase());
        out
    }

    #[cfg(not(target_os = "macos"))]
    fn display_name_other(&self) -> String {
        let mut parts: Vec<String> = self
            .modifiers
            .active_in_order()
            .map(|m| match m {
                // Control and Secondary both render as "Ctrl" on non-macOS,
                // but are output independently to preserve information when
                // both flags are set (matching serializer behaviour).
                Modifier::Control | Modifier::Secondary => "Ctrl".to_owned(),
                Modifier::Alt => "Alt".to_owned(),
                Modifier::Shift => "Shift".to_owned(),
            })
            .collect();

        parts.push(self.key.to_uppercase());
        parts.join("+")
    }
}

impl fmt::Display for Keystroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
