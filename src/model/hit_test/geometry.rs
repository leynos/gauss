//! Geometry helpers for shared hit-testing.
//!
//! These helpers are kept in a private submodule so the public `hit_test`
//! module stays focused on API contracts and deterministic target ordering.

#![expect(
    clippy::float_arithmetic,
    reason = "geometry helpers intentionally use floating-point maths"
)]

use crate::model::{
    Anchor, CubicSegment, SegmentKind, Shape, Vec2, cubic_point, shape_world_bounds,
};

pub(super) fn hit_test_shape_bbox(shape: &Shape, cursor_world: Vec2, tolerance_world: f32) -> bool {
    let Some(bounds) = shape_world_bounds(shape) else {
        return false;
    };

    bounds.contains(cursor_world, tolerance_world)
}

pub(super) fn find_best_segment_hit(
    shape: &Shape,
    cursor_world: Vec2,
    tolerance_squared: f32,
) -> Option<usize> {
    let mut best_segment: Option<(usize, f32)> = None;
    for (seg_index, kind) in shape.path.segments.iter().enumerate() {
        let Some(start) = shape.path.anchors.get(seg_index) else {
            break;
        };
        let Some(end) = shape.path.anchors.get(seg_index + 1) else {
            break;
        };

        let distance_squared = segment_distance_squared(cursor_world, *kind, start, end);

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

    best_segment.map(|(seg_index, _)| seg_index)
}

fn segment_distance_squared(
    cursor_world: Vec2,
    kind: SegmentKind,
    start: &Anchor,
    end: &Anchor,
) -> f32 {
    match kind {
        SegmentKind::Line => point_segment_distance_squared(cursor_world, start.pos, end.pos),
        SegmentKind::Cubic => {
            let c1 = start.handle_out.unwrap_or(start.pos);
            let c2 = end.handle_in.unwrap_or(end.pos);
            cubic_distance_squared(cursor_world, CubicSegment::new(start.pos, c1, c2, end.pos))
        }
    }
}

fn point_segment_distance_squared(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b.sub(a);
    let ap = p.sub(a);
    let denom = ab.magnitude_squared();
    if denom <= f32::EPSILON {
        return p.distance_squared(a);
    }

    let raw_t = ((ap.x * ab.x) + (ap.y * ab.y)) / denom;
    let clamped_t = raw_t.clamp(0.0, 1.0);
    let closest = a.add(ab.mul(clamped_t));
    p.distance_squared(closest)
}

fn cubic_distance_squared(p: Vec2, cubic: CubicSegment) -> f32 {
    // `cubic_distance_squared` uses `STEPS = 16` as a practical balance
    // between hit-test accuracy and pointer-move performance. Increase
    // `STEPS` here for higher-fidelity sampling, or decrease it for faster
    // but less precise cubic hit testing.
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
