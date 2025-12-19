//! Hit-testing support for Phase 0 manipulate mode.
//!
//! Phase 0's editor model is intentionally simple, so we use a few pragmatic
//! hit-tests:
//!
//! - handles (if present) are hit-tested first,
//! - then anchors,
//! - then a loose shape bounding-box check.

use crate::model::{Document, SegmentKind, Shape, ShapeId, Vec2};

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
    pub(super) shape_id: ShapeId,
    pub(super) seg_index: usize,
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
    doc.shapes.iter().rev().find_map(|shape| {
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
) -> Option<ShapeId> {
    doc.shapes.iter().rev().find_map(|shape| {
        hit_test_shape_bbox(shape, cursor_world, tolerance_world).then_some(shape.id)
    })
}

fn hit_test_shape_bbox(shape: &Shape, cursor_world: Vec2, tolerance_world: f32) -> bool {
    let Some((min, max)) = shape_world_bbox(shape) else {
        return false;
    };

    cursor_world.x >= (min.x - tolerance_world)
        && cursor_world.x <= (max.x + tolerance_world)
        && cursor_world.y >= (min.y - tolerance_world)
        && cursor_world.y <= (max.y + tolerance_world)
}

fn shape_world_bbox(shape: &Shape) -> Option<(Vec2, Vec2)> {
    let mut bounds = Bounds::new();

    for anchor in &shape.path.anchors {
        bounds.update(anchor.pos);
    }

    for (seg_index, kind) in shape.path.segments.iter().enumerate() {
        let Some(start_anchor) = shape.path.anchors.get(seg_index) else {
            break;
        };
        let Some(end_anchor) = shape.path.anchors.get(seg_index + 1) else {
            break;
        };

        let start = start_anchor.pos;
        let end = end_anchor.pos;

        match kind {
            SegmentKind::Line => {
                bounds.update(start);
                bounds.update(end);
            }
            SegmentKind::Cubic => {
                let c1 = start_anchor.handle_out.unwrap_or(start);
                let c2 = end_anchor.handle_in.unwrap_or(end);
                extend_bounds_with_cubic(&mut bounds, CubicSegment::new(start, c1, c2, end));
            }
        }
    }

    if shape.path.closed
        && let (Some(last), Some(first)) = (shape.path.anchors.last(), shape.path.anchors.first())
    {
        match shape.path.closing_segment {
            SegmentKind::Line => {
                bounds.update(last.pos);
                bounds.update(first.pos);
            }
            SegmentKind::Cubic => {
                let c1 = last.handle_out.unwrap_or(last.pos);
                let c2 = first.handle_in.unwrap_or(first.pos);
                extend_bounds_with_cubic(
                    &mut bounds,
                    CubicSegment::new(last.pos, c1, c2, first.pos),
                );
            }
        }
    }

    bounds.to_tuple()
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: Vec2,
    max: Vec2,
}

impl Bounds {
    const fn new() -> Self {
        Self {
            min: Vec2::new(f32::INFINITY, f32::INFINITY),
            max: Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    const fn update(&mut self, point: Vec2) {
        if point.x < self.min.x {
            self.min.x = point.x;
        }
        if point.y < self.min.y {
            self.min.y = point.y;
        }
        if point.x > self.max.x {
            self.max.x = point.x;
        }
        if point.y > self.max.y {
            self.max.y = point.y;
        }
    }

    const fn to_tuple(self) -> Option<(Vec2, Vec2)> {
        if !self.min.x.is_finite()
            || !self.min.y.is_finite()
            || !self.max.x.is_finite()
            || !self.max.y.is_finite()
        {
            return None;
        }

        Some((self.min, self.max))
    }
}

#[derive(Clone, Copy, Debug)]
struct CubicAxis {
    start: f32,
    control_a: f32,
    control_b: f32,
    end: f32,
}

impl CubicAxis {
    const fn new(start: f32, control_a: f32, control_b: f32, end: f32) -> Self {
        Self {
            start,
            control_a,
            control_b,
            end,
        }
    }
}

fn extend_bounds_with_cubic(bounds: &mut Bounds, cubic: CubicSegment) {
    bounds.update(cubic.p0);
    bounds.update(cubic.p3);

    let mut ts = Vec::with_capacity(4);
    add_cubic_extrema_ts(
        CubicAxis::new(cubic.p0.x, cubic.c1.x, cubic.c2.x, cubic.p3.x),
        &mut ts,
    );
    add_cubic_extrema_ts(
        CubicAxis::new(cubic.p0.y, cubic.c1.y, cubic.c2.y, cubic.p3.y),
        &mut ts,
    );

    for t in ts {
        let point = cubic_point(cubic, t);
        bounds.update(point);
    }
}

fn add_cubic_extrema_ts(axis: CubicAxis, out: &mut Vec<f32>) {
    let coeff_a = -axis.start + (3.0 * axis.control_a) - (3.0 * axis.control_b) + axis.end;
    let coeff_b = 2.0 * (axis.start - (2.0 * axis.control_a) + axis.control_b);
    let coeff_c = axis.control_a - axis.start;
    let epsilon = 1.0e-6;

    if coeff_a.abs() < epsilon {
        if coeff_b.abs() < epsilon {
            return;
        }
        let t = -coeff_c / coeff_b;
        if t > 0.0 && t < 1.0 {
            out.push(t);
        }
        return;
    }

    let discriminant = (coeff_b * coeff_b) - (4.0 * coeff_a * coeff_c);
    if discriminant < 0.0 {
        return;
    }

    let sqrt_disc = discriminant.sqrt();
    let denom = 2.0 * coeff_a;
    let t1 = (-coeff_b + sqrt_disc) / denom;
    let t2 = (-coeff_b - sqrt_disc) / denom;

    if t1 > 0.0 && t1 < 1.0 {
        out.push(t1);
    }
    if t2 > 0.0 && t2 < 1.0 {
        out.push(t2);
    }
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

#[derive(Clone, Copy, Debug)]
struct CubicSegment {
    p0: Vec2,
    c1: Vec2,
    c2: Vec2,
    p3: Vec2,
}

impl CubicSegment {
    const fn new(p0: Vec2, c1: Vec2, c2: Vec2, p3: Vec2) -> Self {
        Self { p0, c1, c2, p3 }
    }
}

fn cubic_distance_squared(p: Vec2, cubic: CubicSegment) -> f32 {
    const STEPS: u8 = 16;
    let mut best = f32::INFINITY;
    let mut previous = cubic.p0;
    for i in 1..=STEPS {
        let t = f32::from(i) / f32::from(STEPS);
        let next = cubic_point(cubic, t);
        let candidate = point_segment_distance_squared(p, previous, next);
        best = best.min(candidate);
        previous = next;
    }

    best
}

fn cubic_point(cubic: CubicSegment, t: f32) -> Vec2 {
    let omt = 1.0 - t;
    let omt2 = omt * omt;
    let t2 = t * t;

    cubic
        .p0
        .mul(omt2 * omt)
        .add(cubic.c1.mul(3.0 * omt2 * t))
        .add(cubic.c2.mul(3.0 * omt * t2))
        .add(cubic.p3.mul(t2 * t))
}
