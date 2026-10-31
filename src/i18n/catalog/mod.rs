//! Message catalog storage and lookup.

use std::collections::HashMap;

use super::{I18nError, Locale, MessageId};
mod localizer;
pub use localizer::Localizer;

/// A message catalog for a specific locale.
///
/// Catalogs map message identifiers to localized strings. Each catalog
/// corresponds to a single locale (e.g., en-GB, fr-FR).
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use gauss::i18n::{Catalog, MessageId};
///
/// let mut messages = HashMap::new();
/// messages.insert("greeting".to_owned(), "Hello".to_owned());
/// let catalog = Catalog::from_messages(messages);
///
/// let msg = catalog.get(&MessageId::from("greeting"));
/// assert_eq!(msg, Some("Hello"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    messages: HashMap<String, String>,
}

impl Catalog {
    /// Create a new catalog from a map of message identifiers to strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use gauss::i18n::Catalog;
    ///
    /// let mut messages = HashMap::new();
    /// messages.insert("key".to_owned(), "value".to_owned());
    /// let catalog = Catalog::from_messages(messages);
    /// ```
    #[must_use]
    pub const fn from_messages(messages: HashMap<String, String>) -> Self {
        Self { messages }
    }

    /// Look up a message by its identifier.
    ///
    /// Returns `Some(&str)` if the message exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use gauss::i18n::{Catalog, MessageId};
    ///
    /// let mut messages = HashMap::new();
    /// messages.insert("found".to_owned(), "Found!".to_owned());
    /// let catalog = Catalog::from_messages(messages);
    ///
    /// assert_eq!(catalog.get(&MessageId::from("found")), Some("Found!"));
    /// assert_eq!(catalog.get(&MessageId::from("missing")), None);
    /// ```
    #[must_use]
    pub fn get(&self, message_id: &MessageId) -> Option<&str> {
        self.messages.get(message_id.as_str()).map(String::as_str)
    }

    /// Create the default English (en-GB) catalog.
    ///
    /// This catalog contains the baseline translations for Gauss.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::{Catalog, MessageId};
    ///
    /// let catalog = Catalog::default_en_gb();
    /// let draw = catalog.get(&MessageId::tool_mode_draw());
    /// assert_eq!(draw, Some("Draw"));
    /// ```
    #[must_use]
    pub fn default_en_gb() -> Self {
        let mut messages = HashMap::new();
        Self::insert_tool_mode_messages(&mut messages);
        Self::insert_chrome_messages(&mut messages);
        Self::insert_tool_tooltip_messages(&mut messages);
        Self::insert_status_messages(&mut messages);
        Self::insert_align_messages(&mut messages);
        Self::insert_style_messages(&mut messages);
        Self::insert_doc_messages(&mut messages);
        Self::insert_a11y_messages(&mut messages);
        Self::from_messages(messages)
    }

    fn insert_tool_mode_messages(messages: &mut HashMap<String, String>) {
        messages.insert("tool_mode.draw".to_owned(), "Draw".to_owned());
        messages.insert("tool_mode.manipulate".to_owned(), "Manipulate".to_owned());
        messages.insert("edge_mode.line".to_owned(), "Line".to_owned());
        messages.insert(
            "edge_mode.bezier_auto".to_owned(),
            "Bezier (auto)".to_owned(),
        );
        messages.insert(
            "tool.status.mode_with_edge".to_owned(),
            "Mode: {tool} ({edge})".to_owned(),
        );
        messages.insert("tool.status.mode".to_owned(), "Mode: {tool}".to_owned());
    }

    fn insert_chrome_messages(messages: &mut HashMap<String, String>) {
        messages.insert("chrome.file.new".to_owned(), "New".to_owned());
        messages.insert("chrome.file.open".to_owned(), "Open".to_owned());
        messages.insert("chrome.file.save".to_owned(), "Save".to_owned());
        messages.insert("chrome.file.export_web".to_owned(), "Export Web".to_owned());
        messages.insert(
            "chrome.titlebar.recent".to_owned(),
            "Open recent project".to_owned(),
        );
        messages.insert("chrome.settings".to_owned(), "Settings".to_owned());
        messages.insert("chrome.edit.undo".to_owned(), "Undo".to_owned());
        messages.insert("chrome.edit.redo".to_owned(), "Redo".to_owned());

        messages.insert("chrome.window.minimize".to_owned(), "Minimize".to_owned());
        messages.insert("chrome.window.maximize".to_owned(), "Maximize".to_owned());
        messages.insert("chrome.window.close".to_owned(), "Close Window".to_owned());
    }

    fn insert_tool_tooltip_messages(messages: &mut HashMap<String, String>) {
        // Tool tooltip strings
        messages.insert("tool.tooltip.select".to_owned(), "Select".to_owned());
        messages.insert("tool.tooltip.draw_path".to_owned(), "Draw Path".to_owned());
        messages.insert(
            "tool.tooltip.draw_curve".to_owned(),
            "Draw Curve".to_owned(),
        );
        messages.insert(
            "tool.tooltip.draw_rectangle".to_owned(),
            "Draw Rectangle".to_owned(),
        );
        messages.insert(
            "tool.tooltip.draw_circle".to_owned(),
            "Draw Circle".to_owned(),
        );
    }

    fn insert_status_messages(messages: &mut HashMap<String, String>) {
        // Status bar strings
        messages.insert("status.zoom_out".to_owned(), "Zoom Out".to_owned());
        messages.insert("status.zoom_in".to_owned(), "Zoom In".to_owned());
        messages.insert("status.zoom_area".to_owned(), "Zoom to Area".to_owned());
        messages.insert("status.snap_grid".to_owned(), "Snap to Grid".to_owned());

        // Status template strings
        messages.insert("status.saved".to_owned(), "Saved: {path}".to_owned());
        messages.insert("status.opened".to_owned(), "Opened: {path}".to_owned());
        messages.insert(
            "status.history_error".to_owned(),
            "History error: {error}".to_owned(),
        );
        messages.insert(
            "status.shell_error".to_owned(),
            "Shell error: {error}".to_owned(),
        );
        messages.insert(
            "status.save_failed".to_owned(),
            "Save failed: {error}".to_owned(),
        );
        messages.insert(
            "status.open_failed".to_owned(),
            "Open failed: {error}".to_owned(),
        );
        messages.insert("status.maximized".to_owned(), " [MAX]".to_owned());
        messages.insert("status.plain_text".to_owned(), "Plain Text".to_owned());
        messages.insert("status.zoom_ratio_1_1".to_owned(), "1:1".to_owned());
    }

    fn insert_align_messages(messages: &mut HashMap<String, String>) {
        // Alignment button strings
        messages.insert("align.left".to_owned(), "Align Left".to_owned());
        messages.insert("align.centre".to_owned(), "Align Centre".to_owned());
        messages.insert("align.right".to_owned(), "Align Right".to_owned());
        messages.insert("align.top".to_owned(), "Align Top".to_owned());
        messages.insert("align.middle".to_owned(), "Align Middle".to_owned());
        messages.insert("align.bottom".to_owned(), "Align Bottom".to_owned());
    }

    fn insert_style_messages(messages: &mut HashMap<String, String>) {
        // Style control strings
        messages.insert("style.stroke".to_owned(), "Stroke".to_owned());
        messages.insert("style.fill".to_owned(), "Fill".to_owned());
        messages.insert(
            "style.stroke_loading".to_owned(),
            "Stroke: (loading)".to_owned(),
        );
        messages.insert(
            "style.fill_loading".to_owned(),
            "Fill: (loading)".to_owned(),
        );
    }

    fn insert_doc_messages(messages: &mut HashMap<String, String>) {
        // Document header strings
        messages.insert("doc.untitled".to_owned(), "untitled".to_owned());
    }

    fn insert_a11y_messages(messages: &mut HashMap<String, String>) {
        for (key, value) in [
            ("a11y.canvas", "Drawing canvas"),
            ("a11y.shape_list", "Shapes"),
            ("a11y.shape_item", "Shape {index}"),
            ("a11y.titlebar", "Window title bar"),
            ("a11y.window_menu", "Window menu"),
            ("a11y.window_minimize", "Minimize window"),
            ("a11y.window_maximize", "Maximize window"),
            ("a11y.window_restore", "Restore window"),
            ("a11y.window_fullscreen", "Toggle fullscreen"),
            ("a11y.window_close", "Close window"),
            ("a11y.window_title", "Gauss"),
        ] {
            messages.insert(key.to_owned(), value.to_owned());
        }
    }
}

#[cfg(test)]
mod tests;
