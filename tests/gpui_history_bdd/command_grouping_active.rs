//! BDD bindings for history operations attempted during an active group.

use gpui::TestAppContext;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;

use super::*;

fn prepare_active_group_operation(
    cx: &mut TestAppContext,
    operation: HistoryOperation,
) -> TestSupportResult<()> {
    reset_state();
    let durable_shell = DurableShell::open_for_tests(cx);
    let shape = shape_id(42);
    let (history_before, document_before) = durable_shell.with_visual(cx, |visual_cx, view| {
        replace_document_for_grouping_test(visual_cx, view, document_with_one_shape(shape));
        visual_cx
            .update(|_window, app| {
                view.update(app, |phase0_shell, _view_cx| {
                    phase0_shell.apply_command_for_tests(move_shape_command(
                        shape,
                        match operation {
                            HistoryOperation::Undo => Vec2::new(3.0, 0.0),
                            HistoryOperation::Redo => Vec2::new(2.0, 1.0),
                        },
                    ))
                })
            })
            .map_err(|error| {
                TestSupportError::expectation(format!("setup move failed: {error}"))
            })?;
        visual_cx.run_until_parked();
        if matches!(operation, HistoryOperation::Redo) {
            simulate_document_undo(visual_cx);
        }
        visual_cx
            .update(|_window, app| {
                view.update(app, |phase0_shell, _view_cx| {
                    phase0_shell.begin_document_command_group_for_tests()
                })
            })
            .map_err(|error| {
                TestSupportError::expectation(format!("begin group failed: {error}"))
            })?;
        visual_cx.run_until_parked();
        Ok((
            read_history_len(visual_cx, view),
            read_document(visual_cx, view),
        ))
    })?;
    with_state(|state| {
        state.shell = Some(durable_shell);
        state.shape = Some(shape);
        state.history_before = Some(history_before);
        state.document_before = Some(document_before);
        state.operation = Some(operation);
    });
    Ok(())
}

#[given("a fresh Phase 0 shell test window prepared for undo during an active group")]
fn prepared_for_undo(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_active_group_operation(cx, HistoryOperation::Undo)
}

#[given("a fresh Phase 0 shell test window prepared for redo during an active group")]
fn prepared_for_redo(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_active_group_operation(cx, HistoryOperation::Redo)
}

fn attempt_operation(cx: &mut TestAppContext) -> TestSupportResult<()> {
    let operation = with_state(|state| state.operation).ok_or_else(|| missing("operation"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |phase0_shell, _view_cx| match operation {
                HistoryOperation::Undo => phase0_shell.undo_document_for_tests(),
                HistoryOperation::Redo => phase0_shell.redo_document_for_tests(),
            });
        });
        visual_cx.run_until_parked();
        with_state(|state| state.error = read_last_history_error(visual_cx, view));
        Ok(())
    })
}

#[when("document undo is attempted while the group is active")]
fn attempt_undo(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    attempt_operation(cx)
}

#[when("document redo is attempted while the group is active")]
fn attempt_redo(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    attempt_operation(cx)
}

#[then("the history error is undo while group active")]
fn history_error_is_undo_while_active() -> Result<(), TestSupportError> {
    expect_history_error(
        HistoryError::UndoWhileGroupActive,
        "undo while group active error",
    )
}

#[then("the history error is redo while group active")]
fn history_error_is_redo_while_active() -> Result<(), TestSupportError> {
    expect_history_error(
        HistoryError::RedoWhileGroupActive,
        "redo while group active error",
    )
}

fn assert_document_and_history_unchanged(
    cx: &mut TestAppContext,
    context: &str,
) -> TestSupportResult<()> {
    let history_before = history_before()?;
    let document_before = with_state(|state| state.document_before.clone())
        .ok_or_else(|| missing("document before operation"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual_history = read_history_len(visual_cx, view);
        if actual_history != history_before {
            return Err(TestSupportError::expectation(format!(
                "expected {context} to preserve history state; expected {history_before}, got {actual_history}"
            )));
        }
        let actual_document = read_document(visual_cx, view);
        if actual_document != document_before {
            return Err(TestSupportError::expectation(format!(
                "expected {context} to leave document unchanged"
            )));
        }
        Ok(())
    })
}

#[then("the document and history are unchanged")]
fn document_and_history_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let operation = with_state(|state| state.operation).ok_or_else(|| missing("operation"))?;
    let context = match operation {
        HistoryOperation::Undo => "undo while grouped",
        HistoryOperation::Redo => "redo while grouped",
    };
    assert_document_and_history_unchanged(cx, context)
}

#[then("closing the group preserves the document and history")]
fn closing_group_preserves_state(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    active_group_remains_closable(cx)?;
    let operation = with_state(|state| state.operation).ok_or_else(|| missing("operation"))?;
    let context = match operation {
        HistoryOperation::Undo => "closing group after failed undo",
        HistoryOperation::Redo => "closing group after failed redo",
    };
    assert_document_and_history_unchanged(cx, context)
}

#[scenario(
    path = "tests/features/history_command_grouping_undo.feature",
    name = "Undo while a command group is active preserves state",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn undo_while_active(#[from(state_cleanup)] _cleanup: StateCleanup) {}

#[scenario(
    path = "tests/features/history_command_grouping_undo.feature",
    name = "Redo while a command group is active preserves state",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn redo_while_active(#[from(state_cleanup)] _cleanup: StateCleanup) {}
