//! Shared geometry helpers for editor features.
//!
//! This module centralizes lightweight cubic Bézier math used by both hit
//! testing and selection overlays. The focus is on clarity and predictable
//! bounds rather than high-performance tessellation.

#![expect(
    clippy::float_arithmetic,
    reason = "floating-point maths is required for geometry calculations"
)]

use crate::model::{SegmentKind, Shape, Vec2};

const CUBIC_EXTREMA_EPSILON: f32 = 1.0e-6;

/// Axis-aligned world-space bounds for a shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeWorldBounds {
    /// Minimum corner (smallest x and y).
    pub min: Vec2,
    /// Maximum corner (largest x and y).
    pub max: Vec2,
}

impl ShapeWorldBounds {
    /// Test whether a point lies within these bounds, expanded by `tolerance`
    /// in every direction.
    #[must_use]
    pub fn contains_with_tolerance(self, point: Vec2, tolerance: f32) -> bool {
        let normalized_tolerance = tolerance.max(0.0);
        point.x >= (self.min.x - normalized_tolerance)
            && point.x <= (self.max.x + normalized_tolerance)
            && point.y >= (self.min.y - normalized_tolerance)
            && point.y <= (self.max.y + normalized_tolerance)
    }
}

/// Axis-aligned bounds in world space.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    min: Option<Vec2>,
    max: Option<Vec2>,
}

impl Bounds {
    pub(crate) const fn new() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub(crate) fn update(&mut self, point: Vec2) {
        let min = self.min.unwrap_or(point);
        let max = self.max.unwrap_or(point);

        self.min = Some(Vec2::new(min.x.min(point.x), min.y.min(point.y)));
        self.max = Some(Vec2::new(max.x.max(point.x), max.y.max(point.y)));
    }

    pub(crate) const fn to_tuple(self) -> Option<(Vec2, Vec2)> {
        match (self.min, self.max) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        }
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

/// Compute axis-aligned world-space bounds for a shape.
///
/// The bounds include cubic Bézier extrema, so the result is suitable for UI
/// overlays and hit-test broad phases that need to match the visible curve.
#[must_use]
pub fn shape_world_bounds(shape: &Shape) -> Option<ShapeWorldBounds> {
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
            SegmentKind::Line => {}
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
            SegmentKind::Line => {}
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

    let (min, max) = bounds.to_tuple()?;
    Some(ShapeWorldBounds { min, max })
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

    if coeff_a.abs() < CUBIC_EXTREMA_EPSILON {
        if coeff_b.abs() < CUBIC_EXTREMA_EPSILON {
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
