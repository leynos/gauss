//! Test helpers for creating shape fixtures.

use gauss::model::{Anchor, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2};
use gauss::test_helpers::shape_id_from_seed;

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
