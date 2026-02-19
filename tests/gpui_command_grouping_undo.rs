//! GPUI headless integration tests for document command grouping.

mod common;

use common::{
    assert_vec2_close, ensure_initial_draw, init_test_app, read_document, read_history_len,
    simulate_document_undo,
};
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

#[gpui::test]
fn grouped_document_commands_collapse_to_one_undo_step(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
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

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let history_before = read_history_len(visual_cx, &view);
    let error = visual_cx.update(|_window, app| {
        view.update(app, |shell, _view_cx| {
            shell.end_document_command_group_for_tests()
        })
    });

    assert_eq!(
        error.expect_err("expected closing without begin to fail"),
        "Cannot end command group: no active group",
    );
    assert_eq!(
        read_history_len(visual_cx, &view),
        history_before,
        "expected failed grouping boundary call to leave history unchanged",
    );
}
