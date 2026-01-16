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

use gpui::{Entity, TestAppContext, VisualTestContext};

use crate::model::{Command, ShapeMovement, Vec2};

use super::Phase0Shell;

/// Helper to set up a shell, configure it via a closure, and read `file_status_line()`.
///
/// Reduces boilerplate for tests that only need a single state configuration.
fn with_phase0_shell_status<F>(cx: &mut TestAppContext, configure: F) -> Option<String>
where
    F: FnOnce(&mut Phase0Shell),
{
    cx.update(crate::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            configure(shell);
        });
    });
    visual_cx.run_until_parked();

    visual_cx.read(|app| view.read(app).file_status_line())
}

/// Test case for `file_status_line` precedence verification.
struct StatusLineTestCase {
    name: &'static str,
    history_error: Option<&'static str>,
    save_error: Option<&'static str>,
    open_error: Option<&'static str>,
    saved_path: Option<&'static str>,
    opened_path: Option<&'static str>,
    expected: Option<&'static str>,
}

/// Test cases for status line precedence verification.
const STATUS_LINE_TEST_CASES: &[StatusLineTestCase] = &[
    StatusLineTestCase {
        name: "history error takes priority",
        history_error: Some("undo failed"),
        save_error: Some("disk full"),
        open_error: Some("missing file"),
        saved_path: Some("/tmp/out.svg"),
        opened_path: Some("/tmp/in.svg"),
        expected: Some("History error: undo failed"),
    },
    StatusLineTestCase {
        name: "save error takes priority over open error and paths",
        history_error: None,
        save_error: Some("disk full"),
        open_error: Some("missing file"),
        saved_path: Some("/tmp/out.svg"),
        opened_path: Some("/tmp/in.svg"),
        expected: Some("Save failed: disk full"),
    },
    StatusLineTestCase {
        name: "open error takes priority over paths",
        history_error: None,
        save_error: None,
        open_error: Some("missing file"),
        saved_path: Some("/tmp/out.svg"),
        opened_path: None,
        expected: Some("Open failed: missing file"),
    },
    StatusLineTestCase {
        name: "saved path shown when no errors",
        history_error: None,
        save_error: None,
        open_error: None,
        saved_path: Some("/tmp/out.svg"),
        opened_path: None,
        expected: Some("Saved: /tmp/out.svg"),
    },
    StatusLineTestCase {
        name: "opened path as fallback",
        history_error: None,
        save_error: None,
        open_error: None,
        saved_path: None,
        opened_path: Some("/tmp/in.svg"),
        expected: Some("Opened: /tmp/in.svg"),
    },
    StatusLineTestCase {
        name: "returns None when empty",
        history_error: None,
        save_error: None,
        open_error: None,
        saved_path: None,
        opened_path: None,
        expected: None,
    },
];

#[gpui::test]
fn file_status_line_precedence(cx: &mut TestAppContext) {
    for case in STATUS_LINE_TEST_CASES {
        let status = with_phase0_shell_status(cx, |shell| {
            shell.last_history_error = case.history_error.map(str::to_owned);
            shell.last_save_error = case.save_error.map(str::to_owned);
            shell.last_open_error = case.open_error.map(str::to_owned);
            shell.last_saved_path = case.saved_path.map(PathBuf::from);
            shell.last_opened_path = case.opened_path.map(PathBuf::from);
        });
        assert_eq!(
            status,
            case.expected.map(str::to_owned),
            "case '{}' failed",
            case.name
        );
    }
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

// ============================================================================
// History error propagation tests
//
// These tests exercise the actual error-propagation paths through
// apply_history_operation → apply_history_group → apply_command_for_direction,
// verifying that command failures are properly surfaced via last_history_error.
// ============================================================================

/// Set up a `Phase0Shell` in a test window.
///
/// Initialises the UI and creates a window/view pair for history error tests.
fn setup_phase0_shell(cx: &mut TestAppContext) -> (Entity<Phase0Shell>, &mut VisualTestContext) {
    cx.update(crate::ui::init);
    cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx))
}

/// Update the shell via a closure and run until parked.
fn update_shell<F>(visual_cx: &mut VisualTestContext, view: &Entity<Phase0Shell>, f: F)
where
    F: FnOnce(&mut Phase0Shell) + 'static,
{
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            f(shell);
        });
    });
    visual_cx.run_until_parked();
}

/// Read and clone the last history error from the shell.
fn read_last_history_error(
    visual_cx: &VisualTestContext,
    view: &Entity<Phase0Shell>,
) -> Option<String> {
    visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    })
}

/// History operation type for parameterised tests.
#[derive(Clone, Copy, Debug)]
enum HistoryOp {
    Undo,
    Redo,
}

/// Apply a `MoveShapes` command to the first shape in the document.
///
/// Gets the first shape ID, builds a move command with delta (10.0, 10.0),
/// and applies it via `apply_command_for_tests`.
fn apply_move_to_first_shape(shell: &mut Phase0Shell) {
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
}

/// Assert that a history operation fails with the expected error message.
fn assert_history_op_fails_with_error(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    op: HistoryOp,
) {
    // Apply a MoveShapes command (and undo if testing redo).
    update_shell(visual_cx, view, move |shell| {
        apply_move_to_first_shape(shell);
        if matches!(op, HistoryOp::Redo) {
            shell.undo_document_for_tests();
        }
    });

    // Clear shapes to invalidate the operation.
    update_shell(visual_cx, view, |shell| {
        shell.document_mut_for_tests().shapes.clear();
    });

    // Trigger the history operation—should fail because the shape no longer exists.
    update_shell(visual_cx, view, move |shell| match op {
        HistoryOp::Undo => shell.undo_document_for_tests(),
        HistoryOp::Redo => shell.redo_document_for_tests(),
    });

    // Verify error is surfaced with operation type and error description.
    let error = read_last_history_error(visual_cx, view);
    assert!(error.is_some(), "expected last_history_error to be set");
    let error_msg = error.expect("checked above");
    let expected_prefix = match op {
        HistoryOp::Undo => "Undo failed",
        HistoryOp::Redo => "Redo failed",
    };
    assert!(
        error_msg.contains(expected_prefix),
        "expected '{expected_prefix}' in error: {error_msg}"
    );
    assert!(
        error_msg.contains("Shape not found"),
        "expected 'Shape not found' in error: {error_msg}"
    );
}

/// Verify that undo failures propagate to `last_history_error`.
#[gpui::test]
fn undo_sets_last_history_error_on_failure(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    assert_history_op_fails_with_error(visual_cx, &view, HistoryOp::Undo);
}

/// Verify that redo failures propagate to `last_history_error`.
#[gpui::test]
fn redo_sets_last_history_error_on_failure(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    assert_history_op_fails_with_error(visual_cx, &view, HistoryOp::Redo);
}

/// Assert that a successful history operation clears `last_history_error`.
fn assert_successful_history_op_clears_error(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    op: HistoryOp,
) {
    // Apply a command (and undo if testing redo), then set an error.
    update_shell(visual_cx, view, move |shell| {
        apply_move_to_first_shape(shell);
        if matches!(op, HistoryOp::Redo) {
            shell.undo_document_for_tests();
        }
        shell.last_history_error = Some("simulated prior error".to_owned());
    });

    // Verify the error is present.
    let error_before = read_last_history_error(visual_cx, view);
    assert!(error_before.is_some(), "expected error to be set initially");

    // Trigger the history operation—should succeed and clear the error.
    update_shell(visual_cx, view, move |shell| match op {
        HistoryOp::Undo => shell.undo_document_for_tests(),
        HistoryOp::Redo => shell.redo_document_for_tests(),
    });

    // Verify error is cleared.
    let error_after = read_last_history_error(visual_cx, view);
    assert!(
        error_after.is_none(),
        "expected last_history_error to be cleared after successful {op:?}"
    );
}

/// Verify that successful undo clears `last_history_error`.
#[gpui::test]
fn successful_undo_clears_last_history_error(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    assert_successful_history_op_clears_error(visual_cx, &view, HistoryOp::Undo);
}

/// Verify that successful redo clears `last_history_error`.
#[gpui::test]
fn successful_redo_clears_last_history_error(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    assert_successful_history_op_clears_error(visual_cx, &view, HistoryOp::Redo);
}

/// Verify that `apply_command` clears `last_history_error` on success.
///
/// This ensures that new successful edits clear any stale error from prior
/// failed history operations, providing a recovery path for users.
#[gpui::test]
fn apply_command_clears_last_history_error_on_success(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);

    // Manually set an error to simulate a prior failure.
    update_shell(visual_cx, &view, |shell| {
        shell.last_history_error = Some("simulated prior error".to_owned());
    });

    // Verify the error is present.
    let error_before = read_last_history_error(visual_cx, &view);
    assert!(error_before.is_some(), "expected error to be set initially");

    // Apply a new command—should succeed and clear the error.
    update_shell(visual_cx, &view, apply_move_to_first_shape);

    // Verify error is cleared.
    let error_after = read_last_history_error(visual_cx, &view);
    assert!(
        error_after.is_none(),
        "expected last_history_error to be cleared after successful command"
    );
}
