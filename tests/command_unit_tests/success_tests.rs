//! Success path tests for Result-returning command signatures.
//!
//! These tests verify that commands succeed with valid inputs and that their
//! inverse operations also succeed.

use gauss::model::{
    Command, Document, PaintStyle, ReorderOp, SegmentChange, SegmentKind, ShapeMovement,
    StyleChange, Vec2,
};
use rstest::rstest;
use test_support::shapes::shape_id;

use super::doc_with_two_shapes;

/// Verify `MoveShapes` succeeds and returns a valid inverse for valid shapes.
#[rstest]
fn move_shapes_succeeds_with_valid_shapes(mut doc_with_two_shapes: Document) {
    let target_id = shape_id(1);
    let delta = Vec2::new(5.0, -3.0);
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: target_id,
            delta,
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply should succeed");

    // Verify the inverse can be applied successfully
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("inverse should succeed");
}

/// Verify `SetStyle` succeeds and returns a valid inverse for valid shapes.
#[rstest]
fn set_style_succeeds_with_valid_shapes(mut doc_with_two_shapes: Document) {
    let target_id = shape_id(1);
    let cmd = Command::SetStyle {
        changes: vec![StyleChange {
            shape_id: target_id,
            from: PaintStyle::new(None, 1.0, None),
            to: PaintStyle::new(None, 3.0, None),
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply should succeed");

    // Verify the inverse can be applied successfully
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("inverse should succeed");
}

/// Verify `SetSegmentKind` succeeds and returns a valid inverse for valid segments.
#[rstest]
fn set_segment_kind_succeeds_with_valid_segments(mut doc_with_two_shapes: Document) {
    let target_id = shape_id(1);
    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: target_id,
            segment_index: 0,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: None,
            new_start_handle_out: Some(Vec2::new(1.0, 1.0)),
            old_end_handle_in: None,
            new_end_handle_in: Some(Vec2::new(2.0, 2.0)),
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply should succeed");

    // Verify the inverse can be applied successfully
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("inverse should succeed");
}

/// Verify `Reorder` succeeds with valid shape indices.
#[rstest]
fn reorder_succeeds_with_valid_indices(mut doc_with_two_shapes: Document) {
    let cmd = Command::Reorder {
        operations: vec![ReorderOp {
            shape_id: shape_id(1),
            from_index: 0,
            to_index: 1,
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply should succeed");

    // Verify the shape order changed
    assert_eq!(
        doc_with_two_shapes.shapes.first().map(|s| s.id),
        Some(shape_id(2))
    );
    assert_eq!(
        doc_with_two_shapes.shapes.get(1).map(|s| s.id),
        Some(shape_id(1))
    );

    // Verify the inverse can be applied successfully
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("inverse should succeed");
}
