//! Unit tests for [`DocumentUndoHistory`].

use rstest::rstest;

use crate::model::command::Command;
use crate::model::document::Document;
use crate::model::history::DocumentUndoHistory;
use crate::model::{Anchor, PaintStyle, PathGeom, SegmentKind, Shape, ShapeId, Vec2};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Build a sample shape with a single Line segment between two anchors.
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

/// Convenience: create a document containing one shape, returning both.
fn doc_with_one_shape() -> (Document, ShapeId) {
    let mut doc = Document::new();
    let id = doc.append_shape(sample_shape());
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[rstest]
fn new_history_is_empty() {
    let history = DocumentUndoHistory::new();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[rstest]
fn undo_on_empty_is_noop() {
    let mut history = DocumentUndoHistory::new();
    let mut doc = Document::default();

    history.undo(&mut doc).expect("empty undo should succeed");
    assert!(doc.is_empty());
}

#[rstest]
fn redo_on_empty_is_noop() {
    let mut history = DocumentUndoHistory::new();
    let mut doc = Document::default();

    history.redo(&mut doc).expect("empty redo should succeed");
    assert!(doc.is_empty());
}

#[rstest]
fn record_then_undo_restores_state() {
    let (mut doc, id) = doc_with_one_shape();
    let before = doc.clone();
    let mut history = DocumentUndoHistory::new();

    let (cmd, inverse) = apply_move(&mut doc, id, 5.0, 10.0);
    history.record(cmd, inverse);

    assert!(history.can_undo());
    history.undo(&mut doc).expect("undo should succeed");
    assert_eq!(doc, before);
}

#[rstest]
fn record_then_undo_then_redo_reapplies() {
    let (mut doc, id) = doc_with_one_shape();
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
fn multiple_records_sequential_undo() {
    let (mut doc, id) = doc_with_one_shape();
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
fn clear_empties_history() {
    let (mut doc, id) = doc_with_one_shape();
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
