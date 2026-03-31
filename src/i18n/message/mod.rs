//! Message identifier types for i18n lookups.
//!
//! # Naming Convention
//!
//! Message keys use dot-separated hierarchical segments for logical grouping.
//! This creates a clear namespace structure and makes related keys easy to discover.
//!
//! ## Guidelines
//!
//! - Use dot-separated segments to represent hierarchy (e.g., `tool_mode.draw`,
//!   `edge_mode.line`, `tool.status.mode_with_edge`)
//! - Prefer consistent dot-separated keys over underscores for separation
//! - Use underscores only for suffixes when needed (e.g., `mode_with_edge`)
//! - Group related functionality under common prefixes:
//!   - `tool_mode.*` for tool mode identifiers
//!   - `edge_mode.*` for edge mode identifiers
//!   - `tool.status.*` for status message templates
//!
//! ## Examples
//!
//! | Function | Key | Purpose |
//! |----------|-----|---------|
//! | `tool_mode_draw()` | `tool_mode.draw` | Draw tool mode label |
//! | `tool_mode_manipulate()` | `tool_mode.manipulate` | Manipulate tool mode label |
//! | `edge_mode_line()` | `edge_mode.line` | Line edge mode label |
//! | `edge_mode_bezier_auto()` | `edge_mode.bezier_auto` | Bezier auto edge mode label |
//! | `tool_status_mode_with_edge()` | `tool.status.mode_with_edge` | Status template with edge |
//! | `tool_status_mode()` | `tool.status.mode` | Status template without edge |

use std::fmt;

/// A stable message identifier for catalog lookups.
///
/// Message identifiers use dot-separated namespaces to organize translations.
/// For example: `"tool_mode.draw"`, `"edge_mode.line"`.
///
/// # Examples
///
/// ```rust
/// use gauss::i18n::MessageId;
///
/// let msg_id = MessageId::from("tool_mode.draw");
/// assert_eq!(msg_id.as_str(), "tool_mode.draw");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MessageId {
    key: String,
}

impl MessageId {
    /// Create a new message identifier from a string key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::MessageId;
    ///
    /// let msg_id = MessageId::new("edge_mode.bezier_auto");
    /// assert_eq!(msg_id.as_str(), "edge_mode.bezier_auto");
    /// ```
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }

    /// Return the key string for this message identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::MessageId;
    ///
    /// let msg_id = MessageId::from("tool_mode.manipulate");
    /// assert_eq!(msg_id.as_str(), "tool_mode.manipulate");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.key
    }
}

impl From<String> for MessageId {
    fn from(key: String) -> Self {
        Self { key }
    }
}

impl From<&str> for MessageId {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}

impl AsRef<str> for MessageId {
    fn as_ref(&self) -> &str {
        &self.key
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

// Include factory methods module
mod factories;

#[cfg(test)]
mod tests;
