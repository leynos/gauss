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

    /// Message identifier for draw tool mode.
    #[must_use]
    pub fn tool_mode_draw() -> Self {
        Self::new("tool_mode.draw")
    }

    /// Message identifier for manipulate tool mode.
    #[must_use]
    pub fn tool_mode_manipulate() -> Self {
        Self::new("tool_mode.manipulate")
    }

    /// Message identifier for line edge mode.
    #[must_use]
    pub fn edge_mode_line() -> Self {
        Self::new("edge_mode.line")
    }

    /// Message identifier for bezier auto edge mode.
    #[must_use]
    pub fn edge_mode_bezier_auto() -> Self {
        Self::new("edge_mode.bezier_auto")
    }

    /// Message identifier for status template with edge mode.
    #[must_use]
    pub fn tool_status_mode_with_edge() -> Self {
        Self::new("tool.status.mode_with_edge")
    }

    /// Message identifier for status template without edge mode.
    #[must_use]
    pub fn tool_status_mode() -> Self {
        Self::new("tool.status.mode")
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

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for i18n message identifier functionality.
    //!
    //! Validates `MessageId` construction, conversion, and factory methods
    //! for tool modes, edge modes, and status templates.

    use super::*;

    #[test]
    fn message_id_new_creates_correctly() {
        let msg_id = MessageId::new("test.message");
        assert_eq!(msg_id.as_str(), "test.message");
    }

    #[test]
    fn message_id_from_string_creates_correctly() {
        let msg_id = MessageId::from("test.key".to_owned());
        assert_eq!(msg_id.as_str(), "test.key");
    }

    #[test]
    fn message_id_from_str_creates_correctly() {
        let msg_id = MessageId::from("test.str");
        assert_eq!(msg_id.as_str(), "test.str");
    }

    #[test]
    fn message_id_tool_mode_draw_is_correct() {
        let msg_id = MessageId::tool_mode_draw();
        assert_eq!(msg_id.as_str(), "tool_mode.draw");
    }

    #[test]
    fn message_id_tool_mode_manipulate_is_correct() {
        let msg_id = MessageId::tool_mode_manipulate();
        assert_eq!(msg_id.as_str(), "tool_mode.manipulate");
    }

    #[test]
    fn message_id_edge_mode_line_is_correct() {
        let msg_id = MessageId::edge_mode_line();
        assert_eq!(msg_id.as_str(), "edge_mode.line");
    }

    #[test]
    fn message_id_edge_mode_bezier_auto_is_correct() {
        let msg_id = MessageId::edge_mode_bezier_auto();
        assert_eq!(msg_id.as_str(), "edge_mode.bezier_auto");
    }

    #[test]
    fn message_id_display_shows_key() {
        let msg_id = MessageId::new("display.test");
        assert_eq!(format!("{msg_id}"), "display.test");
    }

    #[test]
    fn message_id_equality_works() {
        let msg1 = MessageId::tool_mode_draw();
        let msg2 = MessageId::from("tool_mode.draw");
        assert_eq!(msg1, msg2);
    }
}
