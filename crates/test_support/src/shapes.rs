//! Test helpers for creating shape fixtures.

use gauss::model::{Anchor, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2};
use uuid::Uuid;

/// Create a `ShapeId` from a seed value.
///
/// Deterministic ID generation for test fixtures.
#[must_use]
pub fn shape_id(seed: u128) -> ShapeId {
    ShapeId::from(Uuid::from_u128(seed))
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
    }
}
