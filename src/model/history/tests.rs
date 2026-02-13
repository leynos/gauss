//! Unit tests for [`DocumentUndoHistory`].

use rstest::{fixture, rstest};

use crate::model::command::Command;
use crate::model::document::Document;
use crate::model::history::DocumentUndoHistory;
use crate::model::{Anchor, PaintStyle, PathGeom, SegmentKind, Shape, ShapeId, Vec2};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A sample shape with a single Line segment between two anchors.
#[fixture]
fn sample_shape() -> Shape {
    let mut path = PathGeom::new();
    path.anchors.push(Anchor::new(Vec2::new(10.0, 20.0)));
    path.anchors.push(Anchor::new(Vec2::new(30.0, 40.0)));
    path.segments.push(SegmentKind::Line);

    Shape {
        id: ShapeId::default(),
        z: 0,
        style: PaintStyle::new(None, 1.0, None),
        path,
    }
}

/// A document containing one shape, returned alongside the shape's ID.
#[fixture]
fn doc_with_one_shape(sample_shape: Shape) -> (Document, ShapeId) {
    let mut doc = Document::new();
    let id = doc.append_shape(sample_shape);
    (doc, id)
}

/// Build and apply a `MoveShapes` command, returning the command and inverse.
fn apply_move(
    doc: &mut Document,
    shape_id: ShapeId,
    dx: f32,
    dy: f32,
) -> (Command, crate::model::command::CommandInverse) {
    let cmd = Command::MoveShapes {
        movements: vec![crate::model::ShapeMovement {
            shape_id,
            delta: Vec2::new(dx, dy),
        }],
    };
    let inverse = cmd.apply(doc).expect("apply_move should succeed");
    (cmd, inverse)
}

/// Build and apply an `InsertShape` command, returning the command and inverse.
fn apply_insert(
    doc: &mut Document,
    index: usize,
    shape: Shape,
) -> (Command, crate::model::command::CommandInverse) {
    let cmd = Command::InsertShape {
        insertion: crate::model::ShapeInsertion { index, shape },
    };
    let inverse = cmd.apply(doc).expect("apply_insert should succeed");
    (cmd, inverse)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[rstest]
fn new_history_is_empty() {
    let history = DocumentUndoHistory::new();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

/// Undo and redo on an empty history are both no-ops.
#[rstest]
#[case::undo(true)]
#[case::redo(false)]
fn empty_history_operation_is_noop(#[case] is_undo: bool) {
    let mut history = DocumentUndoHistory::new();
    let mut doc = Document::default();

    let result = if is_undo {
        history.undo(&mut doc)
    } else {
        history.redo(&mut doc)
    };

    result.expect("empty history operation should succeed");
    assert!(doc.is_empty());
}

#[rstest]
fn record_then_undo_restores_state(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let before = doc.clone();
    let mut history = DocumentUndoHistory::new();

    let (cmd, inverse) = apply_move(&mut doc, id, 5.0, 10.0);
    history.record(cmd, inverse);

    assert!(history.can_undo());
    history.undo(&mut doc).expect("undo should succeed");
    assert_eq!(doc, before);
}

#[rstest]
fn record_then_undo_then_redo_reapplies(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd, inverse) = apply_move(&mut doc, id, 5.0, 10.0);
    let after_move = doc.clone();
    history.record(cmd, inverse);

    history.undo(&mut doc).expect("undo should succeed");
    assert!(history.can_redo());

    history.redo(&mut doc).expect("redo should succeed");
    assert_eq!(doc, after_move);
}

#[rstest]
fn multiple_records_sequential_undo(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let state_0 = doc.clone();
    let mut history = DocumentUndoHistory::new();

    let (cmd1, inv1) = apply_move(&mut doc, id, 1.0, 0.0);
    let state_1 = doc.clone();
    history.record(cmd1, inv1);

    let (cmd2, inv2) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd2, inv2);

    // Undo second command — back to state_1
    history.undo(&mut doc).expect("undo #2 should succeed");
    assert_eq!(doc, state_1);

    // Undo first command — back to state_0
    history.undo(&mut doc).expect("undo #1 should succeed");
    assert_eq!(doc, state_0);
}

#[rstest]
fn clear_empties_history(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd, inverse) = apply_move(&mut doc, id, 3.0, 4.0);
    history.record(cmd, inverse);
    assert!(history.can_undo());

    history.clear();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[rstest]
fn undo_propagates_error_on_bad_shape() {
    let mut doc = Document::new();
    let mut history = DocumentUndoHistory::new();

    // Fabricate a command referencing a non-existent shape so that its
    // inverse will fail when applied.
    let bogus_id = ShapeId::default();
    let cmd = Command::MoveShapes {
        movements: vec![crate::model::ShapeMovement {
            shape_id: bogus_id,
            delta: Vec2::new(1.0, 1.0),
        }],
    };
    let inverse = crate::model::CommandInverse::MoveShapesBack {
        command_name: "Move",
        movements: vec![crate::model::ShapeMovement {
            shape_id: bogus_id,
            delta: Vec2::new(-1.0, -1.0),
        }],
    };
    history.record(cmd, inverse);

    let result = history.undo(&mut doc);
    let msg = result.expect_err("undo should fail for bogus shape");
    assert!(msg.contains("Undo"), "error should mention Undo: {msg}");
    assert!(
        msg.contains("Move"),
        "error should mention command name: {msg}"
    );
}

#[rstest]
fn default_impl_matches_new() {
    let a = DocumentUndoHistory::new();
    let b = DocumentUndoHistory::default();

    assert!(!a.can_undo());
    assert!(!b.can_undo());
}

// ---------------------------------------------------------------------------
// Historical undo (branch-edit) tests
// ---------------------------------------------------------------------------

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
    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0);
    let state_a = doc.clone();
    history.record(cmd_a, inv_a);

    // B: move down
    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd_b, inv_b);

    // Undo B — back to state_a
    history.undo(&mut doc).expect("undo B should succeed");
    assert_eq!(doc, state_a);

    // C: move left (branch edit after undoing B)
    let (cmd_c, inv_c) = apply_move(&mut doc, id, -3.0, 0.0);
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

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd_a, inv_a);

    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd_b, inv_b);

    let (cmd_c, inv_c) = apply_move(&mut doc, id, -1.0, 0.0);
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

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd_a, inv_a);

    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0);
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

    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0);
    let state_after_a = doc.clone();
    history.record(cmd_a, inv_a);

    history.clear();
    history
        .undo(&mut doc)
        .expect("undo after clear should be noop");

    assert_eq!(doc, state_after_a);
}

/// `InsertShape` round-trip through history: insert, undo removes it,
/// redo re-inserts it.
#[rstest]
fn insert_shape_round_trip_through_history(sample_shape: Shape) {
    let mut doc = Document::new();
    let mut history = DocumentUndoHistory::new();

    assert!(doc.is_empty());

    let (cmd, inverse) = apply_insert(&mut doc, 0, sample_shape);
    assert_eq!(doc.len(), 1);
    let original_path = doc.shape_at(0).expect("shape exists").path.clone();
    history.record(cmd, inverse);

    // Undo — shape removed
    history.undo(&mut doc).expect("undo insert should succeed");
    assert!(doc.is_empty());

    // Redo — shape re-inserted
    history.redo(&mut doc).expect("redo insert should succeed");
    assert_eq!(doc.len(), 1);
    assert_eq!(
        doc.shape_at(0).expect("shape exists after redo").path,
        original_path
    );
}
