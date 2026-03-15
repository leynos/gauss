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

#![expect(
    clippy::float_arithmetic,
    reason = "selection overlays use floating-point layout and padding"
)]

use crate::model::{Document, Selection, Shape, Vec2, Viewport, shape_world_bounds};
use crate::ui::selection_utils::selected_shape_ids;

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
        let Some(shape) = doc.shape(shape_id) else {
            continue;
        };

        add_shape_overlays(&mut overlays, shape, viewport);
    }

    overlays
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
    let bounds = shape_world_bounds(shape)?;
    let screen_min = viewport.world_to_screen(bounds.min);
    let screen_max = viewport.world_to_screen(bounds.max);
    Some((screen_min, screen_max))
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
