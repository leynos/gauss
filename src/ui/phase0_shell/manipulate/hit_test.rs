//! Hit-testing support for Phase 0 manipulate mode.
//!
//! Phase 0's editor model is intentionally simple, so we use a few pragmatic
//! hit-tests:
//!
//! - handles (if present) are hit-tested first,
//! - then anchors,
//! - then a loose shape bounding-box check.

use crate::model::{
    CubicSegment, Document, SegmentKind, Shape, ShapeId, Vec2, cubic_point, shape_world_bounds,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct ShapeHit {
    pub(super) shape_index: usize,
    pub(super) shape_id: ShapeId,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AnchorHit {
    pub(super) shape_index: usize,
    pub(super) shape_id: ShapeId,
    pub(super) anchor_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HandleHitKind {
    In,
    Out,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct HandleHit {
    pub(super) shape_index: usize,
    pub(super) shape_id: ShapeId,
    pub(super) anchor_index: usize,
    pub(super) kind: HandleHitKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SegmentHit {
    pub(super) shape_index: usize,
    pub(super) shape_id: ShapeId,
    pub(super) seg_index: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum MouseDownHit {
    Handle(HandleHit),
    Anchor(AnchorHit),
    Segment(SegmentHit),
    Shape(ShapeHit),
    None,
}

pub(super) fn hit_under_cursor(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> MouseDownHit {
    if let Some(hit) = hit_test_topmost_handle(doc, cursor_world, tolerance_world) {
        return MouseDownHit::Handle(hit);
    }

    if let Some(hit) = hit_test_topmost_anchor(doc, cursor_world, tolerance_world) {
        return MouseDownHit::Anchor(hit);
    }

    if let Some(hit) = hit_test_topmost_segment(doc, cursor_world, tolerance_world) {
        return MouseDownHit::Segment(hit);
    }

    hit_test_topmost_shape(doc, cursor_world, tolerance_world)
        .map_or(MouseDownHit::None, MouseDownHit::Shape)
}

pub(super) fn hit_test_topmost_handle(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<HandleHit> {
    let tolerance_squared = tolerance_world * tolerance_world;
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(shape_index, shape)| {
            shape
                .path
                .anchors
                .iter()
                .enumerate()
                .find_map(|(anchor_index, anchor)| {
                    if anchor
                        .handle_in
                        .is_some_and(|p| p.distance_squared(cursor_world) <= tolerance_squared)
                    {
                        return Some(HandleHit {
                            shape_index,
                            shape_id: shape.id,
                            anchor_index,
                            kind: HandleHitKind::In,
                        });
                    }

                    if anchor
                        .handle_out
                        .is_some_and(|p| p.distance_squared(cursor_world) <= tolerance_squared)
                    {
                        return Some(HandleHit {
                            shape_index,
                            shape_id: shape.id,
                            anchor_index,
                            kind: HandleHitKind::Out,
                        });
                    }

                    None
                })
        })
}

pub(super) fn hit_test_topmost_segment(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<SegmentHit> {
    let tolerance_squared = tolerance_world * tolerance_world;
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(shape_index, shape)| {
            let mut best_segment: Option<(usize, f32)> = None;
            for (seg_index, kind) in shape.path.segments.iter().enumerate() {
                let Some(start) = shape.path.anchors.get(seg_index) else {
                    break;
                };
                let Some(end) = shape.path.anchors.get(seg_index + 1) else {
                    break;
                };

                let distance_squared = match kind {
                    SegmentKind::Line => {
                        point_segment_distance_squared(cursor_world, start.pos, end.pos)
                    }
                    SegmentKind::Cubic => {
                        let c1 = start.handle_out.unwrap_or(start.pos);
                        let c2 = end.handle_in.unwrap_or(end.pos);
                        cubic_distance_squared(
                            cursor_world,
                            CubicSegment::new(start.pos, c1, c2, end.pos),
                        )
                    }
                };

                if distance_squared > tolerance_squared {
                    continue;
                }

                match best_segment {
                    None => best_segment = Some((seg_index, distance_squared)),
                    Some((_, best)) if distance_squared < best => {
                        best_segment = Some((seg_index, distance_squared));
                    }
                    Some(_) => {}
                }
            }

            best_segment.map(|(seg_index, _)| SegmentHit {
                shape_index,
                shape_id: shape.id,
                seg_index,
            })
        })
}

pub(super) fn hit_test_topmost_anchor(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<AnchorHit> {
    let tolerance_squared = tolerance_world * tolerance_world;
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(shape_index, shape)| {
            shape
                .path
                .anchors
                .iter()
                .enumerate()
                .find_map(|(anchor_index, anchor)| {
                    (anchor.pos.distance_squared(cursor_world) <= tolerance_squared).then_some(
                        AnchorHit {
                            shape_index,
                            shape_id: shape.id,
                            anchor_index,
                        },
                    )
                })
        })
}

pub(super) fn hit_test_topmost_shape(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<ShapeHit> {
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(shape_index, shape)| {
            hit_test_shape_bbox(shape, cursor_world, tolerance_world).then_some(ShapeHit {
                shape_index,
                shape_id: shape.id,
            })
        })
}

fn hit_test_shape_bbox(shape: &Shape, cursor_world: Vec2, tolerance_world: f32) -> bool {
    let Some(bounds) = shape_world_bounds(shape) else {
        return false;
    };

    bounds.contains(cursor_world, tolerance_world)
}

fn point_segment_distance_squared(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b.sub(a);
    let ap = p.sub(a);
    let denom = ab.distance_squared(Vec2::ZERO);
    if denom <= f32::EPSILON {
        return p.distance_squared(a);
    }

    let raw_t = ((ap.x * ab.x) + (ap.y * ab.y)) / denom;
    let clamped_t = raw_t.clamp(0.0, 1.0);
    let closest = a.add(ab.mul(clamped_t));
    p.distance_squared(closest)
}

fn cubic_distance_squared(p: Vec2, cubic: CubicSegment) -> f32 {
    const STEPS: u8 = 16;
    let mut best = f32::INFINITY;
    let mut previous = cubic.start;
    for i in 1..=STEPS {
        let t = f32::from(i) / f32::from(STEPS);
        let next = cubic_point(cubic, t);
        let candidate = point_segment_distance_squared(p, previous, next);
        best = best.min(candidate);
        previous = next;
    }

    best
}
