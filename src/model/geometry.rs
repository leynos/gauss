//! Shared geometry helpers for editor features.
//!
//! This module centralises lightweight cubic Bézier math used by both hit
//! testing and selection overlays. The focus is on clarity and predictable
//! bounds rather than high-performance tessellation.

use crate::model::{SegmentKind, Shape, Vec2};

/// Axis-aligned bounds in world space.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    min: Vec2,
    max: Vec2,
}

impl Bounds {
    pub(crate) const fn new() -> Self {
        Self {
            min: Vec2::new(f32::INFINITY, f32::INFINITY),
            max: Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub(crate) const fn update(&mut self, point: Vec2) {
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

    pub(crate) const fn min(&self) -> Vec2 {
        self.min
    }

    pub(crate) const fn max(&self) -> Vec2 {
        self.max
    }

    pub(crate) const fn contains(self, point: Vec2, tolerance: f32) -> bool {
        point.x >= (self.min.x - tolerance)
            && point.x <= (self.max.x + tolerance)
            && point.y >= (self.min.y - tolerance)
            && point.y <= (self.max.y + tolerance)
    }

    pub(crate) const fn is_valid(self) -> bool {
        self.min.x.is_finite()
            && self.min.y.is_finite()
            && self.max.x.is_finite()
            && self.max.y.is_finite()
    }
}

/// A cubic Bézier segment in world space.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CubicSegment {
    pub(crate) start: Vec2,
    pub(crate) control_a: Vec2,
    pub(crate) control_b: Vec2,
    pub(crate) end: Vec2,
}

impl CubicSegment {
    pub(crate) const fn new(start: Vec2, control_a: Vec2, control_b: Vec2, end: Vec2) -> Self {
        Self {
            start,
            control_a,
            control_b,
            end,
        }
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

/// Evaluate a cubic Bézier segment at parameter `t` (0..=1).
pub(crate) fn cubic_point(cubic: CubicSegment, t: f32) -> Vec2 {
    let one_minus_t = 1.0 - t;
    let one_minus_t_sq = one_minus_t * one_minus_t;
    let t_sq = t * t;
    let coeff_start = one_minus_t_sq * one_minus_t;
    let coeff_a = 3.0 * one_minus_t_sq * t;
    let coeff_b = 3.0 * one_minus_t * t_sq;
    let coeff_end = t_sq * t;

    Vec2::new(
        coeff_start * cubic.start.x
            + coeff_a * cubic.control_a.x
            + coeff_b * cubic.control_b.x
            + coeff_end * cubic.end.x,
        coeff_start * cubic.start.y
            + coeff_a * cubic.control_a.y
            + coeff_b * cubic.control_b.y
            + coeff_end * cubic.end.y,
    )
}

pub(crate) fn shape_world_bounds(shape: &Shape) -> Option<Bounds> {
    let mut bounds = Bounds::new();

    for anchor in &shape.path.anchors {
        bounds.update(anchor.pos);
    }

    for (index, kind) in shape.path.segments.iter().enumerate() {
        let Some(start_anchor) = shape.path.anchors.get(index) else {
            break;
        };
        let Some(end_anchor) = shape.path.anchors.get(index + 1) else {
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

    if bounds.is_valid() {
        Some(bounds)
    } else {
        None
    }
}

fn extend_bounds_with_cubic(bounds: &mut Bounds, cubic: CubicSegment) {
    bounds.update(cubic.start);
    bounds.update(cubic.end);

    let mut ts = Vec::with_capacity(4);
    add_cubic_extrema_ts(
        CubicAxis::new(
            cubic.start.x,
            cubic.control_a.x,
            cubic.control_b.x,
            cubic.end.x,
        ),
        &mut ts,
    );
    add_cubic_extrema_ts(
        CubicAxis::new(
            cubic.start.y,
            cubic.control_a.y,
            cubic.control_b.y,
            cubic.end.y,
        ),
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
