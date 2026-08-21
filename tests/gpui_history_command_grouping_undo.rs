//! GPUI headless integration tests for document command grouping.

#[path = "gpui_history_bdd/command_grouping.rs"]
mod command_grouping;

#[path = "common/gpui_history_command_grouping_undo.rs"]
mod common;

#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open_for_tests.rs"]
mod history_bdd_support_open_for_tests;

use common::{read_document, read_history_len, simulate_document_undo};
use gauss::model::history::HistoryError;
use gauss::model::{Command, Document, ShapeId, ShapeMovement, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Entity, VisualTestContext};
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
