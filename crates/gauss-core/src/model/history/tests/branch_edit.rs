//! Historical undo (branch-edit) tests for [`DocumentUndoHistory`].

use rstest::rstest;

use crate::model::ShapeId;
use crate::model::document::Document;
use crate::model::history::DocumentUndoHistory;

use super::{apply_move, doc_with_one_shape};

/// Do A, do B, undo B, do C — then undo walks through C and the
/// historical branch containing B.
///
/// This validates `undo_2`'s key behavioural difference from classical
/// redo truncation: B is not discarded after the branch edit.
#[rstest]
fn branch_edit_preserves_historical_undo(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let state_0 = doc.clone();
    let mut history = DocumentUndoHistory::new();

    // A: move right
    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0).expect("move A should apply");
    let state_a = doc.clone();
    history.record(cmd_a, inv_a);

    // B: move down
    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0).expect("move B should apply");
    history.record(cmd_b, inv_b);

    // Undo B — back to state_a
    history.undo(&mut doc).expect("undo B should succeed");
    assert_eq!(doc, state_a);

    // C: move left (branch edit after undoing B)
    let (cmd_c, inv_c) = apply_move(&mut doc, id, -3.0, 0.0).expect("move C should apply");
    history.record(cmd_c, inv_c);
    assert!(
        !history.can_redo(),
        "redo should be unavailable after branch edit"
    );

    // Undo C — back to state_a
    history.undo(&mut doc).expect("undo C should succeed");
    assert_eq!(doc, state_a);

    // Undo the historical branch — this redoes B (undo_2 brings it back)
    history.undo(&mut doc).expect("undo branch should succeed");

    // Undo B again
    history
        .undo(&mut doc)
        .expect("undo B-replay should succeed");

    // Undo A — back to state_0
    history.undo(&mut doc).expect("undo A should succeed");
    assert_eq!(doc, state_0);
}

/// Do A, B, C then undo C, undo B, redo B, redo C — state matches
/// the state after all three commands.
#[rstest]
fn multiple_undo_then_redo_round_trip(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0).expect("move A should apply");
    history.record(cmd_a, inv_a);

    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0).expect("move B should apply");
    history.record(cmd_b, inv_b);

    let (cmd_c, inv_c) = apply_move(&mut doc, id, -1.0, 0.0).expect("move C should apply");
    let state_abc = doc.clone();
    history.record(cmd_c, inv_c);

    // Undo C, then B
    history.undo(&mut doc).expect("undo C");
    history.undo(&mut doc).expect("undo B");

    // Redo B, then C
    history.redo(&mut doc).expect("redo B");
    history.redo(&mut doc).expect("redo C");

    assert_eq!(doc, state_abc);
}

/// Do A, B then undo B, clear — history is fully reset.
#[rstest]
fn clear_after_partial_undo(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0).expect("move A should apply");
    history.record(cmd_a, inv_a);

    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0).expect("move B should apply");
    history.record(cmd_b, inv_b);

    history.undo(&mut doc).expect("undo B");

    history.clear();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

/// Do A, clear, undo — document is unchanged (undo is a no-op).
#[rstest]
fn undo_after_clear_is_noop(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0).expect("move A should apply");
    let state_after_a = doc.clone();
    history.record(cmd_a, inv_a);

    history.clear();
    history
        .undo(&mut doc)
        .expect("undo after clear should be noop");

    assert_eq!(doc, state_after_a);
}
