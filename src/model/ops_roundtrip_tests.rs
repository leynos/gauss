//! Unit tests for `DocOp`/`DocChange` inversion behaviour.
//!
//! These tests validate the core undo/redo invariant for the model layer:
//! applying an operation and then applying its inverse restores the document.

#![allow(
    clippy::expect_used,
    reason = "unit tests use expect for clear failure messages"
)]

use crate::model::{
    Anchor, DocChange, DocOp, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId,
    Vec2,
};
use rstest::rstest;
use uuid::Uuid;

#[must_use]
fn shape_id(seed: u128) -> ShapeId {
    ShapeId::from(Uuid::from_u128(seed))
}

#[must_use]
fn sample_shape(id: ShapeId, z: i32) -> Shape {
    let mut path = PathGeom::new();
    path.anchors.push(Anchor::new(Vec2::new(10.0, 20.0)));
    path.anchors.push(Anchor {
        pos: Vec2::new(30.0, 40.0),
        handle_in: Some(Vec2::new(25.0, 35.0)),
        handle_out: None,
    });
    path.segments.push(SegmentKind::Line);

    Shape {
        id,
        z,
        style: PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), 2.0, None),
        path,
    }
}

#[must_use]
fn sample_doc() -> Document {
    Document {
        shapes: vec![sample_shape(shape_id(1), 0), sample_shape(shape_id(2), 1)],
    }
}

fn assert_round_trip(mut doc: Document, op: &DocOp) {
    let before = doc.clone();
    op.apply(&mut doc);
    op.invert().apply(&mut doc);
    assert_eq!(doc, before);
}

#[rstest]
fn insert_shape_inverts() {
    let mut doc = sample_doc();
    let shape = sample_shape(shape_id(3), 2);
    let op = DocOp::InsertShape { index: 1, shape };

    let before = doc.clone();
    op.apply(&mut doc);
    op.invert().apply(&mut doc);
    assert_eq!(doc, before);
}

#[rstest]
fn remove_shape_inverts() {
    let doc = sample_doc();
    let shape = doc
        .shapes
        .first()
        .cloned()
        .expect("expected sample document to contain a shape");
    assert_round_trip(doc, &DocOp::RemoveShape { index: 0, shape });
}

#[rstest]
fn move_anchor_inverts() {
    let doc = sample_doc();
    assert_round_trip(
        doc,
        &DocOp::MoveAnchor {
            shape: shape_id(1),
            anchor: 0,
            from: Vec2::new(10.0, 20.0),
            to: Vec2::new(11.0, 21.0),
        },
    );
}

#[rstest]
fn move_handle_in_inverts() {
    let doc = sample_doc();
    assert_round_trip(
        doc,
        &DocOp::MoveHandleIn {
            shape: shape_id(1),
            anchor: 0,
            from: None,
            to: Some(Vec2::new(5.0, 6.0)),
        },
    );
}

#[rstest]
fn move_handle_out_inverts() {
    let doc = sample_doc();
    assert_round_trip(
        doc,
        &DocOp::MoveHandleOut {
            shape: shape_id(2),
            anchor: 0,
            from: None,
            to: Some(Vec2::new(50.0, 60.0)),
        },
    );
}

#[rstest]
fn move_shape_inverts() {
    let doc = sample_doc();
    assert_round_trip(
        doc,
        &DocOp::MoveShape {
            shape: shape_id(1),
            delta: Vec2::new(3.0, -2.0),
        },
    );
}

#[rstest]
fn set_style_inverts() {
    let doc = sample_doc();
    let from = PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), 2.0, None);
    let to = PaintStyle::new(None, 5.0, Some(Rgba::new(0, 0, 255, 255)));
    assert_round_trip(
        doc,
        &DocOp::SetStyle {
            shape: shape_id(1),
            from,
            to,
        },
    );
}

#[rstest]
fn set_segment_kind_inverts() {
    let doc = sample_doc();
    assert_round_trip(
        doc,
        &DocOp::SetSegmentKind {
            shape: shape_id(1),
            seg: 0,
            from: SegmentKind::Line,
            to: SegmentKind::Cubic,
        },
    );
}

#[rstest]
fn reorder_inverts() {
    let doc = sample_doc();
    assert_round_trip(
        doc,
        &DocOp::Reorder {
            shape: shape_id(1),
            from: 0,
            to: 1,
        },
    );
}

#[rstest]
fn doc_change_apply_then_inverse_restores_document() {
    let mut doc = sample_doc();
    let before = doc.clone();

    let change = DocChange {
        ops: vec![
            DocOp::MoveShape {
                shape: shape_id(1),
                delta: Vec2::new(4.0, 7.0),
            },
            DocOp::SetSegmentKind {
                shape: shape_id(2),
                seg: 0,
                from: SegmentKind::Line,
                to: SegmentKind::Cubic,
            },
        ],
    };

    change.apply(&mut doc);
    change.apply_inverse(&mut doc);

    assert_eq!(doc, before);
}
