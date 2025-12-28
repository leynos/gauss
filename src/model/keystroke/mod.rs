//! Platform-independent keystroke representation.
//!
//! This module provides types for representing keyboard shortcuts without
//! depending on GPUI, enabling testing and serialization of keybindings.
//!
//! # Design
//!
//! The [`Keystroke`] type captures the key and modifiers for a keyboard
//! shortcut. It provides:
//!
//! - Builder methods for ergonomic construction
//! - Conversion to GPUI keystroke format via [`Keystroke::to_gpui_string`]
//! - Human-readable display for UI and documentation
//!
//! # Platform Modifiers
//!
//! The [`Modifiers`] type uses a `secondary` flag for the platform-specific
//! "command" modifier (Cmd on macOS, Ctrl on other platforms). This matches
//! GPUI's cross-platform keystroke handling.
//!
//! # Examples
//!
//! ```rust,no_run
//! use gauss::model::Keystroke;
//!
//! // Simple key
//! let tab = Keystroke::new("tab");
//! assert_eq!(tab.to_gpui_string(), "tab");
//!
//! // Platform "secondary" modifier (Cmd on macOS, Ctrl elsewhere)
//! let undo = Keystroke::secondary("z");
//! // On macOS: "cmd-z", on Linux/Windows: "ctrl-z"
//!
//! // Multiple modifiers
//! let redo = Keystroke::secondary_shift("z");
//! ```

#[cfg(test)]
mod tests;

use std::fmt;

/// Modifier keys for a keystroke.
///
/// Modifiers track which modifier keys are held during a keystroke. The
/// `secondary` field represents the platform-specific "command" modifier
/// (Cmd on macOS, Ctrl on other platforms).
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::Modifiers;
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
    /// "command" modifier, which GPUI renders as `cmd-` on macOS and
    /// `ctrl-` on other platforms.
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
}

/// A platform-independent keystroke representation.
///
/// This type captures keyboard shortcuts without GPUI dependency, enabling
/// testing and serialization of keybindings.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::Keystroke;
///
/// // Simple key
/// let delete = Keystroke::new("backspace");
///
/// // With modifiers
/// let select_all = Keystroke::secondary("a");
/// let deselect = Keystroke::secondary_shift("a");
///
/// // Convert to GPUI format
/// assert_eq!(select_all.to_gpui_string(), "cmd-a"); // macOS
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
    /// use gauss::model::Keystroke;
    ///
    /// let tab = Keystroke::new("tab");
    /// assert_eq!(tab.to_gpui_string(), "tab");
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
    /// use gauss::model::Keystroke;
    ///
    /// let undo = Keystroke::secondary("z");
    /// // On macOS: "cmd-z", on Linux/Windows: "ctrl-z"
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
    /// use gauss::model::Keystroke;
    ///
    /// let redo = Keystroke::secondary_shift("z");
    /// // On macOS: "shift-cmd-z", on Linux/Windows: "shift-ctrl-z"
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
    /// use gauss::model::Keystroke;
    ///
    /// let close = Keystroke::alt("f4");
    /// assert_eq!(close.to_gpui_string(), "alt-f4");
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
    /// use gauss::model::{Keystroke, Modifiers};
    ///
    /// let mods = Modifiers::default().with_control().with_shift();
    /// let keystroke = Keystroke::with_modifiers("a", mods);
    /// assert_eq!(keystroke.to_gpui_string(), "ctrl-shift-a");
    /// ```
    #[must_use]
    pub fn with_modifiers(key: impl Into<String>, modifiers: Modifiers) -> Self {
        Self {
            key: key.into(),
            modifiers,
        }
    }

    /// Convert to a GPUI-compatible keystroke string.
    ///
    /// The format follows GPUI conventions:
    ///
    /// - Modifiers are lowercase and hyphen-separated
    /// - Order: ctrl, alt, shift, cmd/secondary
    /// - Key is lowercase
    ///
    /// Note: The `secondary` modifier is rendered as `cmd` on macOS and `ctrl`
    /// on other platforms at runtime. This method always outputs `cmd` for
    /// `secondary` because GPUI handles the platform mapping internally.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::Keystroke;
    ///
    /// assert_eq!(Keystroke::new("tab").to_gpui_string(), "tab");
    /// assert_eq!(Keystroke::secondary("z").to_gpui_string(), "cmd-z");
    /// assert_eq!(Keystroke::secondary_shift("a").to_gpui_string(), "shift-cmd-a");
    /// assert_eq!(Keystroke::alt("f4").to_gpui_string(), "alt-f4");
    /// ```
    #[must_use]
    pub fn to_gpui_string(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.control {
            parts.push("ctrl");
        }
        if self.modifiers.alt {
            parts.push("alt");
        }
        if self.modifiers.shift {
            parts.push("shift");
        }
        // GPUI uses "cmd" for the secondary modifier regardless of platform;
        // the platform mapping happens at runtime in GPUI.
        if self.modifiers.secondary {
            parts.push("cmd");
        }

        parts.push(&self.key);
        parts.join("-")
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
    /// use gauss::model::Keystroke;
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
        let mut parts = Vec::new();

        if self.modifiers.control {
            parts.push("⌃");
        }
        if self.modifiers.alt {
            parts.push("⌥");
        }
        if self.modifiers.shift {
            parts.push("⇧");
        }
        if self.modifiers.secondary {
            parts.push("⌘");
        }

        parts.push(&self.key.to_uppercase());
        parts.concat()
    }

    #[cfg(not(target_os = "macos"))]
    fn display_name_other(&self) -> String {
        let mut parts = Vec::new();

        // Output control and secondary independently to preserve information
        // when both are set (matching to_gpui_string behaviour)
        if self.modifiers.control {
            parts.push("Ctrl".to_owned());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_owned());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_owned());
        }
        // Secondary maps to Ctrl on non-macOS platforms
        if self.modifiers.secondary {
            parts.push("Ctrl".to_owned());
        }

        parts.push(self.key.to_uppercase());
        parts.join("+")
    }
}

impl fmt::Display for Keystroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
