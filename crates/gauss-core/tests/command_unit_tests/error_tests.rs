//! Error condition tests for commands (Issue #28).
//!
//! These tests verify that commands return appropriate errors when operating
//! on non-existent shapes, anchors, or segments.

use gauss_core::model::{
    Anchor, AnchorMovement, Command, Document, HandleKind, HandleMovement, PaintStyle, ReorderOp,
    SegmentChange, SegmentKind, ShapeMovement, StyleChange, UserError, Vec2,
};
use rstest::rstest;
use test_support::shapes::shape_id;

use super::{
    assert_fails_with_anchor_not_found, assert_fails_with_invalid_operation,
    assert_fails_with_segment_not_found, assert_fails_with_shape_not_found, doc_with_two_shapes,
    empty_doc,
};

/// Verify `MoveShapes` returns an error for non-existent shapes.
#[rstest]
fn move_shapes_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: missing_id,
            delta: Vec2::new(10.0, 10.0),
        }],
    };

    assert_fails_with_shape_not_found(empty_doc, &cmd, missing_id);
}

/// Verify `MoveAnchor` returns an error for non-existent shapes.
#[rstest]
fn move_anchor_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: missing_id,
            anchor_index: 0,
            original: Anchor {
                pos: Vec2::new(0.0, 0.0),
                handle_in: None,
                handle_out: None,
            },
            delta: Vec2::new(10.0, 10.0),
        },
    };

    assert_fails_with_shape_not_found(empty_doc, &cmd, missing_id);
}

/// Verify `MoveAnchor` returns an error for non-existent anchors.
#[rstest]
fn move_anchor_fails_for_missing_anchor(doc_with_two_shapes: Document) {
    let target_shape = shape_id(1);
    let missing_anchor = 999;
    let cmd = Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: target_shape,
            anchor_index: missing_anchor,
            original: Anchor {
                pos: Vec2::new(0.0, 0.0),
                handle_in: None,
                handle_out: None,
            },
            delta: Vec2::new(10.0, 10.0),
        },
    };

    assert_fails_with_anchor_not_found(doc_with_two_shapes, &cmd, target_shape, missing_anchor);
}

/// Verify `MoveHandle` returns an error for non-existent shapes.
#[rstest]
fn move_handle_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: missing_id,
            anchor_index: 0,
            kind: HandleKind::In,
            from: None,
            to: Some(Vec2::new(10.0, 10.0)),
        },
    };

    assert_fails_with_shape_not_found(empty_doc, &cmd, missing_id);
}

/// Verify `MoveHandle` returns an error for non-existent anchors.
#[rstest]
fn move_handle_fails_for_missing_anchor(doc_with_two_shapes: Document) {
    let target_shape = shape_id(1);
    let missing_anchor = 999;
    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: target_shape,
            anchor_index: missing_anchor,
            kind: HandleKind::In,
            from: None,
            to: Some(Vec2::new(10.0, 10.0)),
        },
    };

    assert_fails_with_anchor_not_found(doc_with_two_shapes, &cmd, target_shape, missing_anchor);
}

/// Verify `SetStyle` returns an error for non-existent shapes.
#[rstest]
fn set_style_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::SetStyle {
        changes: vec![StyleChange {
            shape_id: missing_id,
            from: PaintStyle::new(None, 1.0, None),
            to: PaintStyle::new(None, 2.0, None),
        }],
    };

    assert_fails_with_shape_not_found(empty_doc, &cmd, missing_id);
}

/// Verify `Reorder` returns an error for invalid indices.
#[rstest]
fn reorder_fails_for_invalid_indices(empty_doc: Document) {
    let cmd = Command::Reorder {
        operations: vec![ReorderOp {
            shape_id: shape_id(999),
            from_index: 0,
            to_index: 1,
        }],
    };

    assert_fails_with_invalid_operation(empty_doc, &cmd, "invalid reorder");
}

/// Verify `SetSegmentKind` returns an error for non-existent shapes.
#[rstest]
fn set_segment_kind_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: missing_id,
            segment_index: 0,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: None,
            new_start_handle_out: Some(Vec2::new(1.0, 1.0)),
            old_end_handle_in: None,
            new_end_handle_in: Some(Vec2::new(2.0, 2.0)),
        }],
    };

    assert_fails_with_shape_not_found(empty_doc, &cmd, missing_id);
}

/// Verify `SetSegmentKind` returns an error for non-existent segments.
#[rstest]
fn set_segment_kind_fails_for_missing_segment(doc_with_two_shapes: Document) {
    let target_shape = shape_id(1);
    let missing_segment = 999;
    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: target_shape,
            segment_index: missing_segment,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: None,
            new_start_handle_out: Some(Vec2::new(1.0, 1.0)),
            old_end_handle_in: None,
            new_end_handle_in: Some(Vec2::new(2.0, 2.0)),
        }],
    };

    assert_fails_with_segment_not_found(doc_with_two_shapes, &cmd, target_shape, missing_segment);
}

/// Verify that `CommandInverse::apply` propagates errors when the document
/// has been mutated into an invalid state after the original command applied.
///
/// This ensures undo paths fail loudly instead of corrupting document state.
#[rstest]
fn inverse_apply_fails_when_document_state_changes(mut doc_with_two_shapes: Document) {
    // Apply a valid MoveShapes command and capture the inverse.
    let target_id = shape_id(1);
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: target_id,
            delta: Vec2::new(10.0, 10.0),
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("command should apply successfully");

    // Mutate the document into an invalid state: remove the shape the inverse
    // expects to operate on.
    doc_with_two_shapes
        .remove_shape_by_id(target_id)
        .expect("expected to remove target shape");

    // Applying the inverse should now fail instead of silently corrupting state.
    let err = inverse
        .apply(&mut doc_with_two_shapes)
        .expect_err("expected ShapeNotFound error");
    match err {
        UserError::ShapeNotFound(id) => assert_eq!(id, target_id),
        other => panic!("unexpected error: {other:?}"),
    }
}
