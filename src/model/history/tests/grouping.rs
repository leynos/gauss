//! Command grouping tests for [`DocumentUndoHistory`].

use rstest::rstest;

use crate::model::ShapeId;
use crate::model::document::Document;
use crate::model::history::DocumentUndoHistory;

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
    assert_eq!(err, "Cannot begin command group: group already active");

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
    assert_eq!(err, "Cannot end command group: no active group");
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
    assert_eq!(err, "Cannot end command group: no active group");
}
