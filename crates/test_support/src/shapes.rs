//! Test helpers for creating shape fixtures.

use gauss_core::model::{Anchor, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2};
use gauss_core::test_helpers::shape_id_from_seed;

/// Create a `ShapeId` from a seed value.
///
/// Deterministic ID generation for test fixtures.
#[must_use]
pub fn shape_id(seed: u128) -> ShapeId {
    shape_id_from_seed(seed)
}

/// Create a sample shape with the given ID and z-order.
///
/// Returns a simple shape with a two-anchor path suitable for testing.
#[must_use]
pub fn sample_shape(id: ShapeId, z: i32) -> Shape {
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
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Create an open triangle shape with three anchors and two line segments.
///
/// Useful for testing path-closing and segment-kind workflows.
#[must_use]
pub fn open_triangle(id: ShapeId, z: i32) -> Shape {
    let mut path = PathGeom::new();
    path.anchors.push(Anchor::new(Vec2::new(0.0, 0.0)));
    path.anchors.push(Anchor::new(Vec2::new(10.0, 0.0)));
    path.anchors.push(Anchor::new(Vec2::new(5.0, 10.0)));
    path.segments.push(SegmentKind::Line);
    path.segments.push(SegmentKind::Line);

    Shape {
        id,
        z,
        style: PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), 2.0, None),
        path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Create a sample shape with two anchors and cubic handles.
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
        style: PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), 2.0, None),
        path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Create a deterministic two-point line shape for BDD fixtures.
#[must_use]
pub fn line_shape(seed: u128) -> Shape {
    Shape {
        id: shape_id_from_seed(seed),
        z: 0,
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(0.0, 0.0)),
                Anchor::new(Vec2::new(10.0, 10.0)),
            ],
            segments: vec![SegmentKind::Line],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}
