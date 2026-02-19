//! Command grouping tests for [`DocumentUndoHistory`].

use rstest::rstest;

use crate::model::document::Document;
use crate::model::history::{
    DocumentUndoHistory, GROUPING_ERROR_GROUP_ALREADY_ACTIVE, GROUPING_ERROR_NO_ACTIVE_GROUP,
    GROUPING_ERROR_REDO_WHILE_GROUP_ACTIVE, GROUPING_ERROR_UNDO_WHILE_GROUP_ACTIVE,
};
use crate::model::{Command, CommandInverse, ReorderOp, ShapeId, ShapeMovement, Vec2};

use super::{apply_move, doc_with_one_shape};

#[rstest]
fn grouped_commands_collapse_to_one_entry_and_undo_redo_as_a_batch(
    doc_with_one_shape: (Document, ShapeId),
) {
    let (mut doc, id) = doc_with_one_shape;
    let state_before_group = doc.clone();
    let mut history = DocumentUndoHistory::new();

    history.begin_group().expect("begin group should succeed");

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd_a, inv_a);
    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd_b, inv_b);

    // Grouped commands are not realized until the group is closed.
    assert_eq!(history.len(), 0);

    let state_after_group = doc.clone();
    history.end_group().expect("end group should succeed");
    assert_eq!(history.len(), 1);

    history.undo(&mut doc).expect("undo should succeed");
    assert_eq!(doc, state_before_group);
    assert_eq!(history.len(), 0);
    assert!(history.can_redo());

    history.redo(&mut doc).expect("redo should succeed");
    assert_eq!(doc, state_after_group);
    assert_eq!(history.len(), 1);
}

#[rstest]
fn empty_group_is_noop() {
    let mut history = DocumentUndoHistory::new();

    history.begin_group().expect("begin group should succeed");
    history.end_group().expect("end empty group should succeed");

    assert_eq!(history.len(), 0);
    assert!(history.is_empty());
    assert!(!history.can_undo());
}

#[rstest]
fn nested_begin_group_returns_deterministic_error() {
    let mut history = DocumentUndoHistory::new();

    history.begin_group().expect("first begin should succeed");
    let err = history
        .begin_group()
        .expect_err("nested begin should return an error");
    assert_eq!(err, GROUPING_ERROR_GROUP_ALREADY_ACTIVE);

    history
        .end_group()
        .expect("active group should remain closable");
}

#[rstest]
fn end_group_without_begin_returns_deterministic_error() {
    let mut history = DocumentUndoHistory::new();

    let err = history
        .end_group()
        .expect_err("end without begin should return an error");
    assert_eq!(err, GROUPING_ERROR_NO_ACTIVE_GROUP);
}

#[rstest]
fn clear_discards_active_group_and_realized_history(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd, inv) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd, inv);
    assert_eq!(history.len(), 1);

    history.begin_group().expect("begin group should succeed");
    let (grouped_cmd, grouped_inv) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(grouped_cmd, grouped_inv);

    history.clear();
    assert_eq!(history.len(), 0);
    assert!(history.is_empty());

    let err = history
        .end_group()
        .expect_err("clear should discard active group");
    assert_eq!(err, GROUPING_ERROR_NO_ACTIVE_GROUP);
}

#[rstest]
fn undo_while_group_is_open_returns_error(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();
    let (cmd, inv) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd, inv);
    let state_before_undo = doc.clone();

    history.begin_group().expect("begin group should succeed");
    let err = history
        .undo(&mut doc)
        .expect_err("undo should fail while a group is active");
    assert_eq!(err, GROUPING_ERROR_UNDO_WHILE_GROUP_ACTIVE);
    assert_eq!(doc, state_before_undo);
    assert!(history.can_undo());
}

#[rstest]
fn redo_while_group_is_open_returns_error(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();
    let (cmd, inv) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd, inv);
    history.undo(&mut doc).expect("undo should succeed");
    let state_before_redo = doc.clone();

    history.begin_group().expect("begin group should succeed");
    let err = history
        .redo(&mut doc)
        .expect_err("redo should fail while a group is active");
    assert_eq!(err, GROUPING_ERROR_REDO_WHILE_GROUP_ACTIVE);
    assert_eq!(doc, state_before_redo);
    assert!(history.can_redo());
}

#[rstest]
fn grouped_redo_reports_first_error_and_leaves_partial_state(
    doc_with_one_shape: (Document, ShapeId),
) {
    let (mut doc, id) = doc_with_one_shape;
    let baseline = doc.clone();
    let mut history = DocumentUndoHistory::new();

    history.begin_group().expect("begin group should succeed");

    // Step 1: will fail on redo ("Do") because the shape id does not exist.
    let first_fail = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: ShapeId::default(),
            delta: Vec2::new(1.0, 0.0),
        }],
    };
    history.record(
        first_fail,
        CommandInverse::MoveShapesBack {
            command_name: "Move",
            movements: vec![],
        },
    );

    // Step 2: succeeds on redo and mutates document state.
    let (middle_cmd, middle_inv) = apply_move(&mut doc, id, 2.0, 0.0);
    history.record(middle_cmd, middle_inv);
    let expected_partial = doc.clone();

    // Step 3: also fails on redo with a distinct command name.
    let second_fail = Command::Reorder {
        operations: vec![ReorderOp {
            shape_id: ShapeId::default(),
            from_index: 0,
            to_index: 1,
        }],
    };
    history.record(
        second_fail,
        CommandInverse::ReverseReorder {
            command_name: "Reorder",
            operations: vec![],
        },
    );

    history.end_group().expect("end group should succeed");

    // Move to an undone cursor position so redo executes the grouped Do steps.
    history.undo(&mut doc).expect("undo should succeed");
    assert_eq!(doc, baseline);

    let err = history
        .redo(&mut doc)
        .expect_err("redo should fail because grouped steps contain failures");
    assert!(
        err.contains("Redo failed for 'Move'"),
        "expected first failure message, got: {err}"
    );
    assert!(
        !err.contains("Redo failed for 'Reorder'"),
        "later failure should not overwrite first message: {err}"
    );
    assert_eq!(doc, expected_partial);
}
