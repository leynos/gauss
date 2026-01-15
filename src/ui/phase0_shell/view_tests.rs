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
//!
//! Also tests error propagation through the history system:
//!
//! - Undo failures are surfaced via `last_history_error`
//! - Redo failures are surfaced via `last_history_error`
//! - Successful operations clear `last_history_error`

use std::path::PathBuf;

use gpui::TestAppContext;

use crate::model::{Command, ShapeMovement, Vec2};

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

// ============================================================================
// History error propagation tests
//
// These tests exercise the actual error-propagation paths through
// apply_history_operation → apply_history_group → apply_command_for_direction,
// verifying that command failures are properly surfaced via last_history_error.
// ============================================================================

/// Verify that undo failures propagate to `last_history_error`.
///
/// This test:
/// 1. Applies a command to build undo history
/// 2. Mutates the document to invalidate the inverse (removes the shape)
/// 3. Triggers undo, which should fail
/// 4. Asserts that `last_history_error` contains the expected error message
#[gpui::test]
fn undo_sets_last_history_error_on_failure(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));

    // Get the shape ID from the demo document and apply a MoveShapes command.
    let shape_id = visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            let id = shell
                .document()
                .shapes
                .first()
                .expect("demo document has at least one shape")
                .id;
            let command = Command::MoveShapes {
                movements: vec![ShapeMovement {
                    shape_id: id,
                    delta: Vec2::new(10.0, 10.0),
                }],
            };
            shell
                .apply_command_for_tests(command)
                .expect("command should apply");
            id
        })
    });
    visual_cx.run_until_parked();

    // Mutate the document to remove the shape, making the inverse invalid.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.document_mut_for_tests().shapes.clear();
        });
    });
    visual_cx.run_until_parked();

    // Trigger undo—should fail because the shape no longer exists.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.undo_document_for_tests();
        });
    });
    visual_cx.run_until_parked();

    // Verify error is surfaced with operation type and error description.
    let error = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(error.is_some(), "expected last_history_error to be set");
    let error_msg = error.expect("checked above");
    assert!(
        error_msg.contains("Undo failed"),
        "expected 'Undo failed' in error: {error_msg}"
    );
    assert!(
        error_msg.contains("Shape not found"),
        "expected 'Shape not found' in error: {error_msg}"
    );

    // Suppress unused variable warning - shape_id is needed to construct the command.
    let _ = shape_id;
}

/// Verify that redo failures propagate to `last_history_error`.
///
/// This test:
/// 1. Applies a command and undoes it to set up the redo stack
/// 2. Mutates the document to invalidate the command (removes the shape)
/// 3. Triggers redo, which should fail
/// 4. Asserts that `last_history_error` contains the expected error message
#[gpui::test]
fn redo_sets_last_history_error_on_failure(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));

    // Get the shape ID, apply a command, then undo to set up redo stack.
    let shape_id = visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            let id = shell
                .document()
                .shapes
                .first()
                .expect("demo document has at least one shape")
                .id;
            let command = Command::MoveShapes {
                movements: vec![ShapeMovement {
                    shape_id: id,
                    delta: Vec2::new(10.0, 10.0),
                }],
            };
            shell
                .apply_command_for_tests(command)
                .expect("command should apply");
            shell.undo_document_for_tests();
            id
        })
    });
    visual_cx.run_until_parked();

    // Verify undo succeeded (no error).
    let error_after_undo = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(
        error_after_undo.is_none(),
        "expected no error after successful undo"
    );

    // Mutate the document to remove the shape, making the redo invalid.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.document_mut_for_tests().shapes.clear();
        });
    });
    visual_cx.run_until_parked();

    // Trigger redo—should fail because the shape no longer exists.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.redo_document_for_tests();
        });
    });
    visual_cx.run_until_parked();

    // Verify error is surfaced with operation type and error description.
    let error = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(error.is_some(), "expected last_history_error to be set");
    let error_msg = error.expect("checked above");
    assert!(
        error_msg.contains("Redo failed"),
        "expected 'Redo failed' in error: {error_msg}"
    );
    assert!(
        error_msg.contains("Shape not found"),
        "expected 'Shape not found' in error: {error_msg}"
    );

    // Suppress unused variable warning - shape_id is needed to construct the command.
    let _ = shape_id;
}

/// Verify that successful undo clears `last_history_error`.
///
/// This test:
/// 1. Manually sets `last_history_error` to simulate a prior failure
/// 2. Applies a command and successfully undoes it
/// 3. Asserts that `last_history_error` is cleared
#[gpui::test]
fn successful_undo_clears_last_history_error(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));

    // Apply a command, then manually set an error to simulate prior failure.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            let id = shell
                .document()
                .shapes
                .first()
                .expect("demo document has at least one shape")
                .id;
            let command = Command::MoveShapes {
                movements: vec![ShapeMovement {
                    shape_id: id,
                    delta: Vec2::new(10.0, 10.0),
                }],
            };
            shell
                .apply_command_for_tests(command)
                .expect("command should apply");
            shell.last_history_error = Some("simulated prior error".to_owned());
        });
    });
    visual_cx.run_until_parked();

    // Verify the error is present.
    let error_before = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(error_before.is_some(), "expected error to be set initially");

    // Trigger undo—should succeed and clear the error.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.undo_document_for_tests();
        });
    });
    visual_cx.run_until_parked();

    // Verify error is cleared.
    let error_after = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(
        error_after.is_none(),
        "expected last_history_error to be cleared after successful undo"
    );
}

/// Verify that successful redo clears `last_history_error`.
///
/// This test:
/// 1. Applies a command, undoes it, and manually sets an error
/// 2. Successfully redoes the command
/// 3. Asserts that `last_history_error` is cleared
#[gpui::test]
fn successful_redo_clears_last_history_error(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));

    // Apply a command and undo it, then manually set an error.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            let id = shell
                .document()
                .shapes
                .first()
                .expect("demo document has at least one shape")
                .id;
            let command = Command::MoveShapes {
                movements: vec![ShapeMovement {
                    shape_id: id,
                    delta: Vec2::new(10.0, 10.0),
                }],
            };
            shell
                .apply_command_for_tests(command)
                .expect("command should apply");
            shell.undo_document_for_tests();
            shell.last_history_error = Some("simulated prior error".to_owned());
        });
    });
    visual_cx.run_until_parked();

    // Verify the error is present.
    let error_before = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(error_before.is_some(), "expected error to be set initially");

    // Trigger redo—should succeed and clear the error.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.redo_document_for_tests();
        });
    });
    visual_cx.run_until_parked();

    // Verify error is cleared.
    let error_after = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(
        error_after.is_none(),
        "expected last_history_error to be cleared after successful redo"
    );
}

/// Verify that `apply_command` clears `last_history_error` on success.
///
/// This ensures that new successful edits clear any stale error from prior
/// failed history operations, providing a recovery path for users.
#[gpui::test]
fn apply_command_clears_last_history_error_on_success(cx: &mut TestAppContext) {
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));

    // Manually set an error to simulate a prior failure.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.last_history_error = Some("simulated prior error".to_owned());
        });
    });
    visual_cx.run_until_parked();

    // Verify the error is present.
    let error_before = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(error_before.is_some(), "expected error to be set initially");

    // Apply a new command—should succeed and clear the error.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            let id = shell
                .document()
                .shapes
                .first()
                .expect("demo document has at least one shape")
                .id;
            let command = Command::MoveShapes {
                movements: vec![ShapeMovement {
                    shape_id: id,
                    delta: Vec2::new(5.0, 5.0),
                }],
            };
            shell
                .apply_command_for_tests(command)
                .expect("command should apply");
        });
    });
    visual_cx.run_until_parked();

    // Verify error is cleared.
    let error_after = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(
        error_after.is_none(),
        "expected last_history_error to be cleared after successful command"
    );
}
