//! Shared geometry helpers for editor features.
//!
//! This module centralizes lightweight cubic Bézier maths used by both hit
//! testing and selection overlays. The focus is on clarity and predictable
//! bounds rather than high-performance tessellation.

#![expect(
    clippy::float_arithmetic,
    reason = "floating-point maths is required for geometry calculations"
)]

use crate::model::{SegmentKind, Shape, Vec2};

/// Treat derivative coefficients below this magnitude as numerically zero.
const CUBIC_EXTREMA_EPSILON: f32 = 1.0e-6;

/// Axis-aligned world-space bounds for a shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeWorldBounds {
    /// Minimum corner (smallest x and y).
    min: Vec2,
    /// Maximum corner (largest x and y).
    max: Vec2,
}

impl ShapeWorldBounds {
    /// Create normalized bounds from two corners.
    ///
    /// The constructor ensures `min ≤ max` on both axes by computing the
    /// component-wise minimum and maximum.
    #[must_use]
    pub const fn new(a: Vec2, b: Vec2) -> Self {
        let min = Vec2::new(
            if a.x < b.x { a.x } else { b.x },
            if a.y < b.y { a.y } else { b.y },
        );
        let max = Vec2::new(
            if a.x > b.x { a.x } else { b.x },
            if a.y > b.y { a.y } else { b.y },
        );
        Self { min, max }
    }

    /// Return the minimum corner.
    #[must_use]
    pub const fn min(self) -> Vec2 {
        self.min
    }

    /// Return the maximum corner.
    #[must_use]
    pub const fn max(self) -> Vec2 {
        self.max
    }

    /// Test whether a point lies within these bounds, expanded by `tolerance`
    /// in every direction.
    ///
    /// Negative tolerance values are clamped to zero, so callers need not
    /// perform their own clamping.
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
    /// Smallest coordinate seen so far on each axis, if any point was added.
    min: Option<Vec2>,
    /// Largest coordinate seen so far on each axis, if any point was added.
    max: Option<Vec2>,
}

impl Bounds {
    /// Start an empty accumulator; its first point establishes both corners.
    /// For example, `to_tuple()` returns `None` until an update adds a point.
    pub(crate) const fn new() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    /// Expand the accumulated rectangle to include `point`.
    /// Adding `(2, 3)` to bounds containing `(0, 1)` moves the maximum corner.
    pub(crate) fn update(&mut self, point: Vec2) {
        let min = self.min.unwrap_or(point);
        let max = self.max.unwrap_or(point);

        self.min = Some(Vec2::new(min.x.min(point.x), min.y.min(point.y)));
        self.max = Some(Vec2::new(max.x.max(point.x), max.y.max(point.y)));
    }

    /// Return the minimum and maximum corners once at least one point exists.
    /// For example, an empty accumulator yields `None` rather than zero bounds.
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
    /// Start anchor of the curve.
    pub(crate) start: Vec2,
    /// Control point leaving the start anchor.
    pub(crate) control_a: Vec2,
    /// Control point approaching the end anchor.
    pub(crate) control_b: Vec2,
    /// End anchor of the curve.
    pub(crate) end: Vec2,
}

impl CubicSegment {
    /// Assemble the four points that define a cubic Bézier curve.
    /// The first and last arguments remain the curve endpoints when evaluated.
    pub(crate) const fn new(start: Vec2, control_a: Vec2, control_b: Vec2, end: Vec2) -> Self {
        Self {
            start,
            control_a,
            control_b,
            end,
        }
    }
}

/// The four scalar coordinates of a cubic curve along one axis.
#[derive(Clone, Copy, Debug)]
struct CubicAxis {
    /// Coordinate at the start anchor.
    start: f32,
    /// Coordinate at the first control point.
    control_a: f32,
    /// Coordinate at the second control point.
    control_b: f32,
    /// Coordinate at the end anchor.
    end: f32,
}

impl CubicAxis {
    /// Store the scalar control values used to solve one-axis extrema.
    /// For example, the x coordinates of a segment form one `CubicAxis`.
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
/// Cubic extrema found by a coefficient-tolerance solver extend the bounds, but
/// the solver may omit interior extrema, so exact curve containment is not
/// guaranteed. When both quadratic and linear derivative coefficients are near
/// zero, no candidate parameters are added from that axis.
///
/// Returns `None` for shapes with no anchors.
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
    Some(ShapeWorldBounds::new(min, max))
}

/// Extend bounds with cubic endpoints and extrema found using a coefficient tolerance.
/// The solver may omit interior extrema, so exact curve containment is not guaranteed.
/// When quadratic and linear derivative coefficients are near zero, no candidate
/// parameters are added from that axis.
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

/// Append parameter values in `(0, 1)` where this axis reaches an extremum.
///
/// Near-linear derivatives are handled with a small coefficient tolerance;
/// roots outside the curve interval do not affect its visible bounds.
/// For example, endpoint roots are omitted because the caller adds endpoints.
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
