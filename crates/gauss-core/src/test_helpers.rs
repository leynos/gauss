//! Test helper utilities for Gauss fixtures.

use crate::model::{Anchor, Paint, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2};

/// The AccessKit test ID version used for deterministic fixtures.
pub const TEST_ID_VERSION: u64 = 0xffff_fffe;

/// Create a deterministic `ShapeId` from a seed value.
///
/// Only the low 32 bits of `seed` are preserved so the derived index fits the
/// AccessKit node-ID index field. Seeds that differ only in upper bits above
/// bit 31 map to the same `ShapeId` by design.
#[must_use]
pub fn shape_id_from_seed(seed: u128) -> ShapeId {
    let masked = seed & u128::from(u32::MAX);
    // Masking constrains the value to the `u32` range; clippy cannot infer it.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "masking limits the value to the u32 range"
    )]
    let idx = masked as u32;
    let raw = (TEST_ID_VERSION << 32) | u64::from(idx);
    ShapeId::from_accesskit_node_id(raw)
}

/// Create the shared stroke and fill style used by geometric test fixtures.
#[must_use]
pub const fn sample_style() -> PaintStyle {
    PaintStyle {
        stroke: Paint::Solid(Rgba::new(16, 32, 64, 255)),
        stroke_width: 2.0,
        fill: Paint::None,
    }
}

/// Create the two-anchor shape used by model operation round-trip tests.
///
/// For example, passing `ShapeId::default()` and `2` creates a shape whose
/// z-order is `2`, with two anchors connected by a line segment.
#[cfg(test)]
#[must_use]
pub(crate) fn sample_shape(id: ShapeId, z: i32) -> Shape {
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

/// Create a test shape with shared default metadata and style values.
#[must_use]
pub const fn test_shape(id: ShapeId, path: PathGeom) -> Shape {
    Shape {
        id,
        z: 0,
        style: sample_style(),
        path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Create a closed square with line segments.
#[must_use]
pub fn square_shape(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
    test_shape(
        id,
        PathGeom {
            anchors: vec![
                Anchor::new(min),
                Anchor::new(Vec2::new(max.x, min.y)),
                Anchor::new(max),
                Anchor::new(Vec2::new(min.x, max.y)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
    )
}

/// Create a square whose first anchor exposes an outgoing handle.
#[must_use]
pub fn square_shape_with_out_handle(
    id: ShapeId,
    min: Vec2,
    max: Vec2,
    handle_offset: Vec2,
) -> Shape {
    let mut shape = square_shape(id, min, max);
    if let Some(first_anchor) = shape.path.anchors.first_mut() {
        first_anchor.handle_out = Some(first_anchor.pos.add(handle_offset));
    }
    shape
}

/// Create an open single-anchor shape with only an incoming handle.
#[must_use]
pub fn handle_in_only_shape(id: ShapeId, pos: Vec2, handle_offset: Vec2) -> Shape {
    let mut anchor = Anchor::new(pos);
    anchor.handle_in = Some(pos.add(handle_offset));

    test_shape(
        id,
        PathGeom {
            anchors: vec![anchor],
            segments: vec![],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
    )
}

/// Create an open two-anchor cubic segment with symmetric handles.
#[must_use]
pub fn cubic_shape(id: ShapeId, start: Vec2, end: Vec2, ctrl_offset: Vec2) -> Shape {
    let mut first_anchor = Anchor::new(start);
    let mut second_anchor = Anchor::new(end);
    first_anchor.handle_out = Some(start.add(ctrl_offset));
    second_anchor.handle_in = Some(end.sub(ctrl_offset));

    test_shape(
        id,
        PathGeom {
            anchors: vec![first_anchor, second_anchor],
            segments: vec![SegmentKind::Cubic],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
    )
}
