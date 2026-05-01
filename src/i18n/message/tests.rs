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
#[case::tool_tooltip_draw_circle(MessageId::tool_tooltip_draw_circle, "tool.tooltip.draw_circle")]
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
#[case::status_shell_error(MessageId::status_shell_error, "status.shell_error")]
#[case::status_save_failed(MessageId::status_save_failed, "status.save_failed")]
#[case::status_open_failed(MessageId::status_open_failed, "status.open_failed")]
#[case::status_maximized(MessageId::status_maximized, "status.maximized")]
#[case::status_plain_text(MessageId::status_plain_text, "status.plain_text")]
#[case::status_zoom_ratio_1_1(MessageId::status_zoom_ratio_1_1, "status.zoom_ratio_1_1")]
// Accessibility
#[case::a11y_canvas(MessageId::a11y_canvas, "a11y.canvas")]
#[case::a11y_shape_list(MessageId::a11y_shape_list, "a11y.shape_list")]
#[case::a11y_shape_item(MessageId::a11y_shape_item, "a11y.shape_item")]
#[case::a11y_titlebar(MessageId::a11y_titlebar, "a11y.titlebar")]
#[case::a11y_window_menu(MessageId::a11y_window_menu, "a11y.window_menu")]
#[case::a11y_window_minimize(MessageId::a11y_window_minimize, "a11y.window_minimize")]
#[case::a11y_window_maximize(MessageId::a11y_window_maximize, "a11y.window_maximize")]
#[case::a11y_window_restore(MessageId::a11y_window_restore, "a11y.window_restore")]
#[case::a11y_window_fullscreen(MessageId::a11y_window_fullscreen, "a11y.window_fullscreen")]
#[case::a11y_window_close(MessageId::a11y_window_close, "a11y.window_close")]
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
