//! Selection overlay geometry for Phase 0.
//!
//! Phase 0 paints the document paths via GPUI's `Canvas` paint callback, which
//! is an imperative API. To keep the overlay logic testable without depending
//! on pixel inspection, this module computes the overlay primitives in screen
//! space (points and line segments) and leaves the actual painting to the
//! canvas renderer.
//!
//! The overlays are intentionally "`PoC` quality": bounding boxes are still
//! computed in a lightweight way, but they now account for cubic Bézier
//! extrema so the box better matches the visible curve.

use crate::model::{Document, SelItem, Selection, Shape, ShapeId, Vec2, Viewport};

/// A line segment in screen-space pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct OverlayLine {
    /// Start point in screen coordinates (pixels).
    pub start: Vec2,
    /// End point in screen coordinates (pixels).
    pub end: Vec2,
}

/// A small marker at a screen-space point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct OverlayMarker {
    /// Marker centre in screen coordinates (pixels).
    pub centre: Vec2,
    /// Marker size in pixels (square edge length).
    pub size: f32,
}

/// Selection overlays for Phase 0.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SelectionOverlays {
    /// Bounding box for each selected shape, represented as four dashed edges.
    pub bbox_edges: Vec<OverlayLine>,
    /// Straight handle connector lines from anchor to each present handle.
    pub handle_lines: Vec<OverlayLine>,
    /// Anchor markers for selected shapes.
    pub anchor_markers: Vec<OverlayMarker>,
    /// Handle markers for selected shapes.
    pub handle_markers: Vec<OverlayMarker>,
}

/// Compute all selection overlays in screen space.
///
/// Overlays are computed for shape selections only. If the selection contains
/// anchors/handles/segments, their parent shapes are not inferred.
#[must_use]
pub(super) fn compute_selection_overlays(
    doc: &Document,
    selection: &Selection,
    viewport: Viewport,
) -> SelectionOverlays {
    let mut overlays = SelectionOverlays::default();
    for shape_id in selected_shape_ids(selection) {
        let Some(shape) = doc.shapes.iter().find(|shape| shape.id == shape_id) else {
            continue;
        };

        add_shape_overlays(&mut overlays, shape, viewport);
    }

    overlays
}

fn selected_shape_ids(selection: &Selection) -> impl Iterator<Item = ShapeId> + '_ {
    selection.items.iter().filter_map(|item| match item {
        SelItem::Shape(id) => Some(*id),
        _ => None,
    })
}

fn add_shape_overlays(overlays: &mut SelectionOverlays, shape: &Shape, viewport: Viewport) {
    let Some((bbox_min, bbox_max)) = shape_screen_bbox(shape, viewport) else {
        return;
    };

    let padding = 3.0;
    let padded_min = Vec2::new(bbox_min.x - padding, bbox_min.y - padding);
    let padded_max = Vec2::new(bbox_max.x + padding, bbox_max.y + padding);

    add_bbox_edges(overlays, padded_min, padded_max);
    add_anchor_and_handle_overlays(overlays, shape, viewport);
}

fn shape_screen_bbox(shape: &Shape, viewport: Viewport) -> Option<(Vec2, Vec2)> {
    let (world_min, world_max) = shape_world_bbox(shape)?;
    let screen_min = viewport.world_to_screen(world_min);
    let screen_max = viewport.world_to_screen(world_max);
    Some((screen_min, screen_max))
}

fn shape_world_bbox(shape: &Shape) -> Option<(Vec2, Vec2)> {
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
            crate::model::SegmentKind::Line => {
                bounds.update(start);
                bounds.update(end);
            }
            crate::model::SegmentKind::Cubic => {
                let c1 = start_anchor.handle_out.unwrap_or(start);
                let c2 = end_anchor.handle_in.unwrap_or(end);
                extend_bounds_with_cubic(&mut bounds, CubicSegment::new(start, c1, c2, end));
            }
        }
    }

    if shape.path.closed
        && let (Some(last), Some(first)) = (shape.path.anchors.last(), shape.path.anchors.first())
    {
        let closing_kind = shape.path.closing_segment;
        match closing_kind {
            crate::model::SegmentKind::Line => {
                bounds.update(last.pos);
                bounds.update(first.pos);
            }
            crate::model::SegmentKind::Cubic => {
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
struct CubicSegment {
    start: Vec2,
    control_a: Vec2,
    control_b: Vec2,
    end: Vec2,
}

impl CubicSegment {
    const fn new(start: Vec2, control_a: Vec2, control_b: Vec2, end: Vec2) -> Self {
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

fn cubic_point(cubic: CubicSegment, t: f32) -> Vec2 {
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

fn add_bbox_edges(overlays: &mut SelectionOverlays, min: Vec2, max: Vec2) {
    let top_left = min;
    let top_right = Vec2::new(max.x, min.y);
    let bottom_right = max;
    let bottom_left = Vec2::new(min.x, max.y);

    overlays.bbox_edges.extend([
        OverlayLine {
            start: top_left,
            end: top_right,
        },
        OverlayLine {
            start: top_right,
            end: bottom_right,
        },
        OverlayLine {
            start: bottom_right,
            end: bottom_left,
        },
        OverlayLine {
            start: bottom_left,
            end: top_left,
        },
    ]);
}

fn add_anchor_and_handle_overlays(
    overlays: &mut SelectionOverlays,
    shape: &Shape,
    viewport: Viewport,
) {
    const ANCHOR_SIZE: f32 = 7.0;
    const HANDLE_SIZE: f32 = 6.0;

    for anchor in &shape.path.anchors {
        let anchor_screen = viewport.world_to_screen(anchor.pos);
        overlays.anchor_markers.push(OverlayMarker {
            centre: anchor_screen,
            size: ANCHOR_SIZE,
        });

        if let Some(handle_in) = anchor.handle_in {
            let handle_screen = viewport.world_to_screen(handle_in);
            overlays.handle_lines.push(OverlayLine {
                start: anchor_screen,
                end: handle_screen,
            });
            overlays.handle_markers.push(OverlayMarker {
                centre: handle_screen,
                size: HANDLE_SIZE,
            });
        }

        if let Some(handle_out) = anchor.handle_out {
            let handle_screen = viewport.world_to_screen(handle_out);
            overlays.handle_lines.push(OverlayLine {
                start: anchor_screen,
                end: handle_screen,
            });
            overlays.handle_markers.push(OverlayMarker {
                centre: handle_screen,
                size: HANDLE_SIZE,
            });
        }
    }
}

#[cfg(test)]
mod tests;
