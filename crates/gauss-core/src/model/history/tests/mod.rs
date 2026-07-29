//! Unit tests for [`DocumentUndoHistory`].

mod branch_edit;
mod depth_limit;
mod grouping;

use rstest::{fixture, rstest};

use crate::model::command::Command;
use crate::model::document::Document;
use crate::model::history::{DocumentUndoHistory, HistoryError};
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
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// A document containing one shape, returned alongside the shape's ID.
#[fixture]
fn doc_with_one_shape(sample_shape: Shape) -> (Document, ShapeId) {
    let mut doc = Document::new();
    let id = doc.append_shape(sample_shape);
    (doc, id)
}

/// Apply `cmd` to `doc`, returning the command alongside its inverse.
///
/// `operation` names the calling helper so a failure identifies which command
/// could not be applied. This is the single panic boundary for these helpers:
/// `.expect()` is disallowed outside test-attributed functions (whitaker
/// `no_expect_outside_tests`), so the `Err` variant is handled explicitly here
/// rather than at each call site.
fn apply_command(
    doc: &mut Document,
    cmd: Command,
    operation: &str,
) -> (Command, crate::model::command::CommandInverse) {
    let inverse = match cmd.apply(doc) {
        Ok(inverse) => inverse,
        Err(error) => panic!("{operation} should succeed: {error:?}"),
    };
    (cmd, inverse)
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
    apply_command(doc, cmd, "apply_move")
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
    apply_command(doc, cmd, "apply_insert")
}

// ---------------------------------------------------------------------------
// Basic tests
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
    // Empty document remains empty after undo/redo on an empty history.
    {
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

    // A pre-populated document is unchanged after undo/redo on an empty history.
    {
        let mut history = DocumentUndoHistory::new();
        let (mut doc, _id) = doc_with_one_shape(sample_shape());
        let original_doc = doc.clone();

        let result = if is_undo {
            history.undo(&mut doc)
        } else {
            history.redo(&mut doc)
        };

        result.expect("empty history operation should succeed");
        assert_eq!(doc, original_doc);
    }
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
    let error = result.expect_err("undo should fail for bogus shape");
    match error {
        HistoryError::UndoReplayFailed {
            command_name,
            reason,
        } => {
            assert_eq!(command_name, "Move");
            assert!(
                !reason.is_empty(),
                "undo failure should include the command failure reason",
            );
        }
        other => panic!("expected UndoReplayFailed, got {other:?}"),
    }
}

#[rstest]
fn default_impl_matches_new() {
    let a = DocumentUndoHistory::new();
    let b = DocumentUndoHistory::default();

    assert!(!a.can_undo());
    assert!(!b.can_undo());
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

#[rstest]
fn new_history_has_zero_len() {
    let history = DocumentUndoHistory::new();

    assert_eq!(history.len(), 0);
    assert!(history.is_empty());
}

#[rstest]
fn record_increments_len(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd1, inv1) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd1, inv1);
    assert_eq!(history.len(), 1);

    let (cmd2, inv2) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd2, inv2);
    assert_eq!(history.len(), 2);
}

#[rstest]
fn undo_decrements_len(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd1, inv1) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd1, inv1);
    let (cmd2, inv2) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd2, inv2);
    assert_eq!(history.len(), 2);

    history.undo(&mut doc).expect("undo should succeed");
    assert_eq!(history.len(), 1);
}

#[rstest]
fn redo_restores_len(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd, inv) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd, inv);
    assert_eq!(history.len(), 1);

    history.undo(&mut doc).expect("undo should succeed");
    assert_eq!(history.len(), 0);

    history.redo(&mut doc).expect("redo should succeed");
    assert_eq!(history.len(), 1);
}

#[rstest]
fn clear_resets_len_to_zero(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::new();

    let (cmd, inv) = apply_move(&mut doc, id, 3.0, 4.0);
    history.record(cmd, inv);
    assert_eq!(history.len(), 1);

    history.clear();
    assert_eq!(history.len(), 0);
    assert!(history.is_empty());
}
