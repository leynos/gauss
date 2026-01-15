//! Unit tests for the Phase 0 shell view.
//!
//! Tests the `file_status_line` display logic, verifying correct precedence:
//!
//! 1. History errors take priority over all other statuses
//! 2. Save errors take priority over all other file statuses
//! 3. Open errors take priority over path displays
//! 4. Saved path is shown when no errors exist
//! 5. Opened path is shown as a fallback
//! 6. Returns `None` when no file operations have occurred

use std::path::PathBuf;

use gpui::TestAppContext;

use super::Phase0Shell;

#[gpui::test]
fn file_status_line_prefers_history_error(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_history_error = Some("undo failed".to_owned());
            shell.last_save_error = Some("disk full".to_owned());
            shell.last_open_error = Some("missing file".to_owned());
            shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
            shell.last_opened_path = Some(PathBuf::from("/tmp/in.svg"));
        });
    });
    visual_cx.run_until_parked();

    let status = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status, Some("History error: undo failed".to_owned()));
}

#[gpui::test]
fn file_status_line_clears_history_error_after_success(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_history_error = Some("undo failed".to_owned());
            shell.last_save_error = Some("disk full".to_owned());
        });
    });
    visual_cx.run_until_parked();

    let status = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status, Some("History error: undo failed".to_owned()));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_history_error = None;
        });
    });
    visual_cx.run_until_parked();

    let status_after = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status_after, Some("Save failed: disk full".to_owned()));
}

#[gpui::test]
fn file_status_line_prefers_save_error(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_save_error = Some("disk full".to_owned());
            shell.last_open_error = Some("missing file".to_owned());
            shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
            shell.last_opened_path = Some(PathBuf::from("/tmp/in.svg"));
        });
    });
    visual_cx.run_until_parked();

    let status = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status, Some("Save failed: disk full".to_owned()));
}

#[gpui::test]
fn file_status_line_falls_back_to_open_error(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_open_error = Some("missing file".to_owned());
            shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
        });
    });
    visual_cx.run_until_parked();

    let status = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status, Some("Open failed: missing file".to_owned()));
}

#[gpui::test]
fn file_status_line_reports_paths_when_no_errors(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
        });
    });
    visual_cx.run_until_parked();

    let status = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status, Some("Saved: /tmp/out.svg".to_owned()));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_saved_path = None;
            shell.last_opened_path = Some(PathBuf::from("/tmp/in.svg"));
        });
    });
    visual_cx.run_until_parked();

    let status_after = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status_after, Some("Opened: /tmp/in.svg".to_owned()));
}

#[gpui::test]
fn file_status_line_returns_none_when_empty(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.run_until_parked();

    let status = visual_cx.read(|app| view.read(app).file_status_line());
    assert_eq!(status, None);
}
