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

    // Window chrome strings

    /// Message identifier for "New" file action.
    #[must_use]
    pub fn chrome_file_new() -> Self {
        Self::new("chrome.file.new")
    }

    /// Message identifier for "Open" file action.
    #[must_use]
    pub fn chrome_file_open() -> Self {
        Self::new("chrome.file.open")
    }

    /// Message identifier for "Save" file action.
    #[must_use]
    pub fn chrome_file_save() -> Self {
        Self::new("chrome.file.save")
    }

    /// Message identifier for "Export Web" action.
    #[must_use]
    pub fn chrome_file_export_web() -> Self {
        Self::new("chrome.file.export_web")
    }

    /// Message identifier for "Open recent project" titlebar text.
    #[must_use]
    pub fn chrome_titlebar_recent() -> Self {
        Self::new("chrome.titlebar.recent")
    }

    /// Message identifier for "Settings" button.
    #[must_use]
    pub fn chrome_settings() -> Self {
        Self::new("chrome.settings")
    }

    /// Message identifier for "Undo" action.
    #[must_use]
    pub fn chrome_edit_undo() -> Self {
        Self::new("chrome.edit.undo")
    }

    /// Message identifier for "Redo" action.
    #[must_use]
    pub fn chrome_edit_redo() -> Self {
        Self::new("chrome.edit.redo")
    }

    // Window control strings

    /// Message identifier for "Minimize" window action.
    #[must_use]
    pub fn chrome_window_minimize() -> Self {
        Self::new("chrome.window.minimize")
    }

    /// Message identifier for "Maximize" window action.
    #[must_use]
    pub fn chrome_window_maximize() -> Self {
        Self::new("chrome.window.maximize")
    }

    /// Message identifier for "Close Window" action.
    #[must_use]
    pub fn chrome_window_close() -> Self {
        Self::new("chrome.window.close")
    }

    // Tool tooltip strings

    /// Message identifier for "Select" tool tooltip.
    #[must_use]
    pub fn tool_tooltip_select() -> Self {
        Self::new("tool.tooltip.select")
    }

    /// Message identifier for "Draw Path" tool tooltip.
    #[must_use]
    pub fn tool_tooltip_draw_path() -> Self {
        Self::new("tool.tooltip.draw_path")
    }

    /// Message identifier for "Draw Curve" tool tooltip.
    #[must_use]
    pub fn tool_tooltip_draw_curve() -> Self {
        Self::new("tool.tooltip.draw_curve")
    }

    /// Message identifier for "Draw Rectangle" tool tooltip.
    #[must_use]
    pub fn tool_tooltip_draw_rectangle() -> Self {
        Self::new("tool.tooltip.draw_rectangle")
    }

    /// Message identifier for "Draw Circle" tool tooltip.
    #[must_use]
    pub fn tool_tooltip_draw_circle() -> Self {
        Self::new("tool.tooltip.draw_circle")
    }

    // Status bar strings

    /// Message identifier for "Zoom Out" button.
    #[must_use]
    pub fn status_zoom_out() -> Self {
        Self::new("status.zoom_out")
    }

    /// Message identifier for "Zoom In" button.
    #[must_use]
    pub fn status_zoom_in() -> Self {
        Self::new("status.zoom_in")
    }

    /// Message identifier for "Zoom to Area" button.
    #[must_use]
    pub fn status_zoom_area() -> Self {
        Self::new("status.zoom_area")
    }

    /// Message identifier for "Snap to Grid" button.
    #[must_use]
    pub fn status_snap_grid() -> Self {
        Self::new("status.snap_grid")
    }

    // Alignment button strings

    /// Message identifier for "Align Left" button.
    #[must_use]
    pub fn align_left() -> Self {
        Self::new("align.left")
    }

    /// Message identifier for "Align Centre" button.
    #[must_use]
    pub fn align_centre() -> Self {
        Self::new("align.centre")
    }

    /// Message identifier for "Align Right" button.
    #[must_use]
    pub fn align_right() -> Self {
        Self::new("align.right")
    }

    /// Message identifier for "Align Top" button.
    #[must_use]
    pub fn align_top() -> Self {
        Self::new("align.top")
    }

    /// Message identifier for "Align Middle" button.
    #[must_use]
    pub fn align_middle() -> Self {
        Self::new("align.middle")
    }

    /// Message identifier for "Align Bottom" button.
    #[must_use]
    pub fn align_bottom() -> Self {
        Self::new("align.bottom")
    }

    // Style control strings

    /// Message identifier for "Stroke" label.
    #[must_use]
    pub fn style_stroke() -> Self {
        Self::new("style.stroke")
    }

    /// Message identifier for "Fill" label.
    #[must_use]
    pub fn style_fill() -> Self {
        Self::new("style.fill")
    }

    /// Message identifier for stroke loading text.
    #[must_use]
    pub fn style_stroke_loading() -> Self {
        Self::new("style.stroke_loading")
    }

    /// Message identifier for fill loading text.
    #[must_use]
    pub fn style_fill_loading() -> Self {
        Self::new("style.fill_loading")
    }

    // Document header strings

    /// Message identifier for "untitled" document.
    #[must_use]
    pub fn doc_untitled() -> Self {
        Self::new("doc.untitled")
    }

    // Status template strings

    /// Message identifier for "Saved: {path}" status.
    #[must_use]
    pub fn status_saved() -> Self {
        Self::new("status.saved")
    }

    /// Message identifier for "Opened: {path}" status.
    #[must_use]
    pub fn status_opened() -> Self {
        Self::new("status.opened")
    }

    /// Message identifier for "History error: {error}" status.
    #[must_use]
    pub fn status_history_error() -> Self {
        Self::new("status.history_error")
    }

    /// Message identifier for "Save failed: {error}" status.
    #[must_use]
    pub fn status_save_failed() -> Self {
        Self::new("status.save_failed")
    }

    /// Message identifier for "Open failed: {error}" status.
    #[must_use]
    pub fn status_open_failed() -> Self {
        Self::new("status.open_failed")
    }

    /// Message identifier for maximized indicator.
    #[must_use]
    pub fn status_maximized() -> Self {
        Self::new("status.maximized")
    }

    // Accessibility strings

    /// Message identifier for canvas accessibility label.
    #[must_use]
    pub fn a11y_canvas() -> Self {
        Self::new("a11y.canvas")
    }

    /// Message identifier for shape list accessibility label.
    #[must_use]
    pub fn a11y_shape_list() -> Self {
        Self::new("a11y.shape_list")
    }

    /// Message identifier for shape item accessibility label template.
    #[must_use]
    pub fn a11y_shape_item() -> Self {
        Self::new("a11y.shape_item")
    }

    /// Message identifier for window title.
    #[must_use]
    pub fn a11y_window_title() -> Self {
        Self::new("a11y.window_title")
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

#[cfg(test)]
mod tests {
    //! Unit tests for i18n message identifier functionality.
    //!
    //! Validates `MessageId` construction, conversion, and factory methods
    //! for tool modes, edge modes, and status templates.

    use rstest::rstest;

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

    #[rstest]
    #[case::tool_mode_draw(MessageId::tool_mode_draw, "tool_mode.draw")]
    #[case::tool_mode_manipulate(MessageId::tool_mode_manipulate, "tool_mode.manipulate")]
    #[case::edge_mode_line(MessageId::edge_mode_line, "edge_mode.line")]
    #[case::edge_mode_bezier_auto(MessageId::edge_mode_bezier_auto, "edge_mode.bezier_auto")]
    #[case::tool_status_mode_with_edge(
        MessageId::tool_status_mode_with_edge,
        "tool.status.mode_with_edge"
    )]
    #[case::tool_status_mode(MessageId::tool_status_mode, "tool.status.mode")]
    // Chrome strings
    #[case::chrome_file_new(MessageId::chrome_file_new, "chrome.file.new")]
    #[case::chrome_file_open(MessageId::chrome_file_open, "chrome.file.open")]
    #[case::chrome_file_save(MessageId::chrome_file_save, "chrome.file.save")]
    #[case::chrome_file_export_web(MessageId::chrome_file_export_web, "chrome.file.export_web")]
    #[case::chrome_titlebar_recent(MessageId::chrome_titlebar_recent, "chrome.titlebar.recent")]
    #[case::chrome_settings(MessageId::chrome_settings, "chrome.settings")]
    #[case::chrome_edit_undo(MessageId::chrome_edit_undo, "chrome.edit.undo")]
    #[case::chrome_edit_redo(MessageId::chrome_edit_redo, "chrome.edit.redo")]
    // Window controls
    #[case::chrome_window_minimize(MessageId::chrome_window_minimize, "chrome.window.minimize")]
    #[case::chrome_window_maximize(MessageId::chrome_window_maximize, "chrome.window.maximize")]
    #[case::chrome_window_close(MessageId::chrome_window_close, "chrome.window.close")]
    // Tool tooltips
    #[case::tool_tooltip_select(MessageId::tool_tooltip_select, "tool.tooltip.select")]
    #[case::tool_tooltip_draw_path(MessageId::tool_tooltip_draw_path, "tool.tooltip.draw_path")]
    #[case::tool_tooltip_draw_curve(MessageId::tool_tooltip_draw_curve, "tool.tooltip.draw_curve")]
    #[case::tool_tooltip_draw_rectangle(
        MessageId::tool_tooltip_draw_rectangle,
        "tool.tooltip.draw_rectangle"
    )]
    #[case::tool_tooltip_draw_circle(
        MessageId::tool_tooltip_draw_circle,
        "tool.tooltip.draw_circle"
    )]
    // Status bar
    #[case::status_zoom_out(MessageId::status_zoom_out, "status.zoom_out")]
    #[case::status_zoom_in(MessageId::status_zoom_in, "status.zoom_in")]
    #[case::status_zoom_area(MessageId::status_zoom_area, "status.zoom_area")]
    #[case::status_snap_grid(MessageId::status_snap_grid, "status.snap_grid")]
    // Alignment
    #[case::align_left(MessageId::align_left, "align.left")]
    #[case::align_centre(MessageId::align_centre, "align.centre")]
    #[case::align_right(MessageId::align_right, "align.right")]
    #[case::align_top(MessageId::align_top, "align.top")]
    #[case::align_middle(MessageId::align_middle, "align.middle")]
    #[case::align_bottom(MessageId::align_bottom, "align.bottom")]
    // Style controls
    #[case::style_stroke(MessageId::style_stroke, "style.stroke")]
    #[case::style_fill(MessageId::style_fill, "style.fill")]
    #[case::style_stroke_loading(MessageId::style_stroke_loading, "style.stroke_loading")]
    #[case::style_fill_loading(MessageId::style_fill_loading, "style.fill_loading")]
    // Document
    #[case::doc_untitled(MessageId::doc_untitled, "doc.untitled")]
    // Status templates
    #[case::status_saved(MessageId::status_saved, "status.saved")]
    #[case::status_opened(MessageId::status_opened, "status.opened")]
    #[case::status_history_error(MessageId::status_history_error, "status.history_error")]
    #[case::status_save_failed(MessageId::status_save_failed, "status.save_failed")]
    #[case::status_open_failed(MessageId::status_open_failed, "status.open_failed")]
    #[case::status_maximized(MessageId::status_maximized, "status.maximized")]
    // Accessibility
    #[case::a11y_canvas(MessageId::a11y_canvas, "a11y.canvas")]
    #[case::a11y_shape_list(MessageId::a11y_shape_list, "a11y.shape_list")]
    #[case::a11y_shape_item(MessageId::a11y_shape_item, "a11y.shape_item")]
    #[case::a11y_window_title(MessageId::a11y_window_title, "a11y.window_title")]
    fn message_id_factory_method_is_correct(
        #[case] factory: fn() -> MessageId,
        #[case] expected: &str,
    ) {
        let msg_id = factory();
        assert_eq!(msg_id.as_str(), expected);
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
