//! Shared helpers for command editing integration tests.

use gauss::model::{Anchor, Document, PaintStyle, PathGeom, SegmentKind, Shape, ShapeId, Vec2};

/// Build a shape with two anchors and cubic handles.
#[must_use]
pub fn shape_with_handles(id: ShapeId) -> Shape {
    let mut path = PathGeom::new();
    path.anchors.push(Anchor {
        pos: Vec2::new(0.0, 0.0),
        handle_in: Some(Vec2::new(-1.0, -1.0)),
        handle_out: Some(Vec2::new(1.0, 1.0)),
    });
    path.anchors.push(Anchor {
        pos: Vec2::new(10.0, 0.0),
        handle_in: Some(Vec2::new(9.0, -1.0)),
        handle_out: Some(Vec2::new(11.0, 1.0)),
    });
    path.segments.push(SegmentKind::Cubic);

    Shape {
        id,
        z: 0,
        style: PaintStyle::new(None, 1.0, None),
        path,
    }
}

/// Build a shape with three anchors and mixed segments.
#[must_use]
pub fn shape_with_three_anchors(id: ShapeId) -> Shape {
    let mut path = PathGeom::new();
    path.anchors.push(Anchor {
        pos: Vec2::new(0.0, 0.0),
        handle_in: None,
        handle_out: Some(Vec2::new(2.0, 0.0)),
    });
    path.anchors.push(Anchor {
        pos: Vec2::new(10.0, 0.0),
        handle_in: Some(Vec2::new(9.0, -1.0)),
        handle_out: Some(Vec2::new(11.0, 1.0)),
    });
    path.anchors.push(Anchor {
        pos: Vec2::new(20.0, 0.0),
        handle_in: Some(Vec2::new(19.0, 0.0)),
        handle_out: None,
    });
    path.segments.push(SegmentKind::Cubic);
    path.segments.push(SegmentKind::Line);

    Shape {
        id,
        z: 0,
        style: PaintStyle::new(None, 1.0, None),
        path,
    }
}

/// Fetch a shape by index from a document.
#[must_use]
pub fn shape_at(doc: &Document, index: usize) -> Option<&Shape> {
    doc.shapes.get(index)
}

/// Fetch an anchor by index from a shape.
#[must_use]
pub fn anchor_at(shape: &Shape, index: usize) -> Option<&Anchor> {
    shape.path.anchors.get(index)
}

/// Fetch a segment kind by index from a shape.
#[must_use]
pub fn segment_at(shape: &Shape, index: usize) -> Option<&SegmentKind> {
    shape.path.segments.get(index)
}

/// Fetch a mutable segment kind by index from a shape.
#[must_use]
pub fn segment_mut(shape: &mut Shape, index: usize) -> Option<&mut SegmentKind> {
    shape.path.segments.get_mut(index)
}
