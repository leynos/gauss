//! Factory methods for creating [`MessageId`] instances.
//!
//! This module provides convenience constructors for all message identifiers
//! used throughout the application. Methods are organized by functional area.

use super::MessageId;

/// Tool mode message identifiers.
impl MessageId {
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

/// Window chrome message identifiers.
impl MessageId {
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
}

/// Tool tooltip message identifiers.
impl MessageId {
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
}

/// Status bar message identifiers.
impl MessageId {
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
}

/// Alignment message identifiers.
impl MessageId {
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
}

/// Style control message identifiers.
impl MessageId {
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
}

/// Document message identifiers.
impl MessageId {
    /// Message identifier for "untitled" document.
    #[must_use]
    pub fn doc_untitled() -> Self {
        Self::new("doc.untitled")
    }
}

/// Accessibility message identifiers.
impl MessageId {
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
