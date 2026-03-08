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

/// Create a closed square with line segments.
#[must_use]
pub fn square_shape(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
    Shape {
        id,
        z: 0,
        style: sample_style(),
        path: PathGeom {
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
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
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
        first_anchor.handle_out = Some(min.add(handle_offset));
    }
    shape
}

/// Create an open single-anchor shape with only an incoming handle.
#[must_use]
pub fn handle_in_only_shape(id: ShapeId, pos: Vec2, handle_offset: Vec2) -> Shape {
    let mut anchor = Anchor::new(pos);
    anchor.handle_in = Some(pos.add(handle_offset));

    Shape {
        id,
        z: 0,
        style: sample_style(),
        path: PathGeom {
            anchors: vec![anchor],
            segments: vec![],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Create an open two-anchor cubic segment with symmetric handles.
#[must_use]
pub fn cubic_shape(id: ShapeId, start: Vec2, end: Vec2, ctrl_offset: Vec2) -> Shape {
    let mut first_anchor = Anchor::new(start);
    let mut second_anchor = Anchor::new(end);
    first_anchor.handle_out = Some(start.add(ctrl_offset));
    second_anchor.handle_in = Some(end.sub(ctrl_offset));

    Shape {
        id,
        z: 0,
        style: sample_style(),
        path: PathGeom {
            anchors: vec![first_anchor, second_anchor],
            segments: vec![SegmentKind::Cubic],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}
