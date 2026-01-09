//! Error tests for inverse operations.
//!
//! These tests verify that command inverse operations return appropriate errors
//! when operating on missing shapes or invalid indices.

use gauss::model::{
    CommandInverse, Document, PaintStyle, SegmentChange, SegmentKind, ShapeReplacement,
    StyleChange, UserError,
};
use rstest::rstest;
use test_support::shapes::{sample_shape, shape_id};

use super::empty_doc;

/// Verify `RestoreStyles` (via inverse) returns an error for missing shapes.
#[rstest]
fn restore_styles_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let inverse = CommandInverse::RestoreStyles {
        command_name: "Set Style",
        changes: vec![StyleChange {
            shape_id: missing_id,
            from: PaintStyle::new(None, 1.0, None),
            to: PaintStyle::new(None, 2.0, None),
        }],
    };

    let Err(err) = inverse.apply(&mut empty_doc.clone()) else {
        panic!("expected error");
    };
    match err {
        UserError::ShapeNotFound(id) => assert_eq!(id, missing_id),
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Verify `RestoreSegmentKinds` (via inverse) returns an error for missing shapes.
#[rstest]
fn restore_segment_kinds_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let inverse = CommandInverse::RestoreSegmentKinds {
        command_name: "Set Segment Kind",
        changes: vec![SegmentChange {
            shape_id: missing_id,
            segment_index: 0,
            old_kind: SegmentKind::Cubic,
            new_kind: SegmentKind::Line,
            old_start_handle_out: None,
            new_start_handle_out: None,
            old_end_handle_in: None,
            new_end_handle_in: None,
        }],
    };

    let Err(err) = inverse.apply(&mut empty_doc.clone()) else {
        panic!("expected error");
    };
    match err {
        UserError::ShapeNotFound(id) => assert_eq!(id, missing_id),
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Verify `RemoveAnchor` (via inverse) returns an error for invalid indices.
#[rstest]
fn remove_anchor_fails_for_invalid_shape_index(empty_doc: Document) {
    let inverse = CommandInverse::RemoveAnchor {
        command_name: "Insert Anchor",
        replacement: ShapeReplacement {
            shape_index: 999,
            old_shape: sample_shape(shape_id(1), 0),
            new_shape: sample_shape(shape_id(1), 0),
        },
    };

    let Err(err) = inverse.apply(&mut empty_doc.clone()) else {
        panic!("expected error");
    };
    match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains("out of range"),
                "expected 'out of range' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
