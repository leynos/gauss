//! GPUI headless integration tests for document command grouping.

mod common;

use common::{
    assert_vec2_close, ensure_initial_draw, init_test_app, read_document, read_history_len,
    simulate_document_undo,
};
use gauss::model::history::HistoryError;
use gauss::model::{Command, Document, ShapeId, ShapeMovement, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Entity, TestAppContext, VisualTestContext};
use test_support::shapes::{sample_shape, shape_id};
use test_support::{TestSupportError, TestSupportResult};

fn document_with_one_shape(shape: ShapeId) -> Document {
    let mut document = Document::new();
    document.append_shape(sample_shape(shape, 0));
    document
}

fn first_anchor_for_shape(
    document: &Document,
    shape: ShapeId,
    context: &str,
) -> TestSupportResult<Vec2> {
    let resolved_shape = document
        .shape(shape)
        .ok_or_else(|| TestSupportError::missing("shape", context))?;
    resolved_shape
        .path
        .anchors
        .first()
        .map(|anchor| anchor.pos)
        .ok_or_else(|| TestSupportError::missing("shape anchor", context))
}

fn move_shape_command(shape: ShapeId, delta: Vec2) -> Command {
    Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: shape,
            delta,
        }],
    }
}

#[derive(Clone, Copy)]
struct GroupedMovePlan {
    shape: ShapeId,
    history_before: usize,
    first_delta: Vec2,
    second_delta: Vec2,
}

fn replace_document_for_grouping_test(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    document: Document,
) {
    let view_for_document = view.clone();
    visual_cx.update(move |_window, app| {
        view_for_document.update(app, |shell, _view_cx| {
            shell.replace_document_for_tests(document);
        });
    });
    visual_cx.run_until_parked();
}

fn apply_grouped_moves_for_test(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    plan: GroupedMovePlan,
) -> TestSupportResult<()> {
    let view_for_group = view.clone();
    let grouped_result = visual_cx.update(move |_window, app| {
        view_for_group.update(app, |shell, _view_cx| -> TestSupportResult<()> {
            shell
                .begin_document_command_group_for_tests()
                .map_err(|error| {
                    TestSupportError::expectation(format!(
                        "begin grouped command transaction failed: {error}"
                    ))
                })?;
            shell
                .apply_command_for_tests(move_shape_command(plan.shape, plan.first_delta))
                .map_err(|error| {
                    TestSupportError::expectation(format!(
                        "first grouped command failed to apply: {error}"
                    ))
                })?;
            shell
                .apply_command_for_tests(move_shape_command(plan.shape, plan.second_delta))
                .map_err(|error| {
                    TestSupportError::expectation(format!(
                        "second grouped command failed to apply: {error}"
                    ))
                })?;
            if shell.document_history_len_for_tests() != plan.history_before {
                return Err(TestSupportError::expectation(
                    "grouped commands should not be realized before end_group",
                ));
            }
            Ok(())
        })
    });
    grouped_result?;
    visual_cx.run_until_parked();
    if read_history_len(visual_cx, view) != plan.history_before {
        return Err(TestSupportError::expectation(
            "history should remain unchanged while group is active",
        ));
    }
    Ok(())
}

fn read_last_history_error(
    visual_cx: &VisualTestContext,
    view: &Entity<Phase0Shell>,
) -> Option<HistoryError> {
    visual_cx.read(|app| view.read(app).last_history_error_typed_for_tests().cloned())
}

#[derive(Clone, Copy)]
enum HistoryOperation {
    Undo,
    Redo,
}

#[gpui::test]
fn grouped_document_commands_collapse_to_one_undo_step(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);

    let shape = shape_id(42);
    replace_document_for_grouping_test(visual_cx, &view, document_with_one_shape(shape));

    let history_before = read_history_len(visual_cx, &view);
    let before_doc = read_document(visual_cx, &view);
    let anchor_before = first_anchor_for_shape(&before_doc, shape, "before grouped command")
        .expect("expected anchor before grouped command");

    let first_delta = Vec2::new(2.0, 0.0);
    let second_delta = Vec2::new(0.0, 3.0);
    apply_grouped_moves_for_test(
        visual_cx,
        &view,
        GroupedMovePlan {
            shape,
            history_before,
            first_delta,
            second_delta,
        },
    )
    .expect("expected grouped command transaction to succeed");

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            shell
                .end_document_command_group_for_tests()
                .expect("expected end group to succeed");
        });
    });
    visual_cx.run_until_parked();

    assert_eq!(
        read_history_len(visual_cx, &view),
        history_before + 1,
        "expected one realized history entry after grouped command commit",
    );

    let after_group_doc = read_document(visual_cx, &view);
    let anchor_after_group =
        first_anchor_for_shape(&after_group_doc, shape, "after grouped command")
            .expect("expected anchor after grouped command");
    assert_vec2_close(
        anchor_after_group,
        anchor_before.add(first_delta).add(second_delta),
        "after grouped command",
    )
    .expect("expected grouped command to move shape by total delta");

    simulate_document_undo(visual_cx);
    let after_undo_doc = read_document(visual_cx, &view);
    let anchor_after_undo = first_anchor_for_shape(&after_undo_doc, shape, "after grouped undo")
        .expect("expected anchor after grouped undo");
    assert_vec2_close(anchor_after_undo, anchor_before, "after grouped undo")
        .expect("expected one undo to restore grouped movement");
}

#[gpui::test]
fn end_group_without_begin_returns_error_and_preserves_history(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);

    let history_before = read_history_len(visual_cx, &view);
    let error = visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            shell.end_document_command_group_for_tests()
        })
    });

    assert_eq!(
        error.expect_err("expected closing without begin to fail"),
        HistoryError::NoActiveGroup,
    );
    assert_eq!(
        read_history_len(visual_cx, &view),
        history_before,
        "expected failed grouping boundary call to leave history unchanged",
    );
}

#[gpui::test]
fn nested_begin_group_returns_error_and_preserves_history(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);

    let history_before = read_history_len(visual_cx, &view);
    let first_begin = visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            shell.begin_document_command_group_for_tests()
        })
    });
    first_begin.expect("expected first begin group call to succeed");
    visual_cx.run_until_parked();

    let second_begin = visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            shell.begin_document_command_group_for_tests()
        })
    });
    assert_eq!(
        second_begin.expect_err("expected nested begin group call to fail"),
        HistoryError::GroupAlreadyActive,
    );
    assert_eq!(
        read_history_len(visual_cx, &view),
        history_before,
        "expected nested begin failure to preserve history state",
    );

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            shell
                .end_document_command_group_for_tests()
                .expect("expected active group to remain closable");
        });
    });
    visual_cx.run_until_parked();
}

#[gpui::test]
fn undo_while_group_active_sets_error_and_preserves_document(cx: &mut TestAppContext) {
    verify_history_operation_while_group_active_fails(cx, HistoryOperation::Undo);
}

#[gpui::test]
fn redo_while_group_active_sets_error_and_preserves_document(cx: &mut TestAppContext) {
    verify_history_operation_while_group_active_fails(cx, HistoryOperation::Redo);
}

fn assert_history_action_succeeds<E>(action: Result<(), E>, context: &str)
where
    E: std::fmt::Display,
{
    if let Err(error) = action {
        panic!("{context}: {error}");
    }
}

fn setup_history_operation_while_group_active_fixture(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    shape: ShapeId,
    operation: HistoryOperation,
) {
    match operation {
        HistoryOperation::Undo => {
            visual_cx.update(|_window, app| {
                view.update(app, |shell, _view_cx| {
                    assert_history_action_succeeds(
                        shell.apply_command_for_tests(move_shape_command(
                            shape,
                            Vec2::new(3.0, 0.0),
                        )),
                        "expected setup move to succeed",
                    );
                    assert_history_action_succeeds(
                        shell.begin_document_command_group_for_tests(),
                        "expected begin group before undo attempt to succeed",
                    );
                });
            });
            visual_cx.run_until_parked();
        }
        HistoryOperation::Redo => {
            visual_cx.update(|_window, app| {
                view.update(app, |shell, _view_cx| {
                    assert_history_action_succeeds(
                        shell.apply_command_for_tests(move_shape_command(
                            shape,
                            Vec2::new(2.0, 1.0),
                        )),
                        "expected setup move to succeed",
                    );
                });
            });
            visual_cx.run_until_parked();
            simulate_document_undo(visual_cx);
            visual_cx.update(|_window, app| {
                view.update(app, |shell, _view_cx| {
                    assert_history_action_succeeds(
                        shell.begin_document_command_group_for_tests(),
                        "expected begin group before redo attempt to succeed",
                    );
                });
            });
            visual_cx.run_until_parked();
        }
    }
}

fn execute_history_operation_under_active_group(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    operation: HistoryOperation,
) {
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| match operation {
            HistoryOperation::Undo => shell.undo_document_for_tests(),
            HistoryOperation::Redo => shell.redo_document_for_tests(),
        });
    });
    visual_cx.run_until_parked();
}

fn close_group_after_failed_history_operation(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    operation: HistoryOperation,
) {
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            let close_context = match operation {
                HistoryOperation::Undo => "expected group to remain closable after failed undo",
                HistoryOperation::Redo => "expected group to remain closable after failed redo",
            };
            assert_history_action_succeeds(
                shell.end_document_command_group_for_tests(),
                close_context,
            );
        });
    });
    visual_cx.run_until_parked();
}

fn verify_history_operation_while_group_active_fails(
    cx: &mut TestAppContext,
    operation: HistoryOperation,
) {
    init_test_app(cx);

    let (view, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);

    let shape = shape_id(42);
    replace_document_for_grouping_test(visual_cx, &view, document_with_one_shape(shape));
    setup_history_operation_while_group_active_fixture(visual_cx, &view, shape, operation);

    let history_before = read_history_len(visual_cx, &view);
    let doc_before_attempt = read_document(visual_cx, &view);
    execute_history_operation_under_active_group(visual_cx, &view, operation);

    let operation_name = match operation {
        HistoryOperation::Undo => "undo",
        HistoryOperation::Redo => "redo",
    };
    assert_eq!(
        read_history_len(visual_cx, &view),
        history_before,
        "expected {operation_name} while grouped to preserve history state",
    );
    assert_eq!(
        read_document(visual_cx, &view),
        doc_before_attempt,
        "expected {operation_name} while grouped to leave document unchanged",
    );
    let expected_error = match operation {
        HistoryOperation::Undo => HistoryError::UndoWhileGroupActive,
        HistoryOperation::Redo => HistoryError::RedoWhileGroupActive,
    };
    assert_eq!(
        read_last_history_error(visual_cx, &view),
        Some(expected_error),
    );

    close_group_after_failed_history_operation(visual_cx, &view, operation);
}
