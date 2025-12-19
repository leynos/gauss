//! Selection overlay geometry for Phase 0.
//!
//! Phase 0 paints the document paths via GPUI's `Canvas` paint callback, which
//! is an imperative API. To keep the overlay logic testable without depending
//! on pixel inspection, this module computes the overlay primitives in screen
//! space (points and line segments) and leaves the actual painting to the
//! canvas renderer.
//!
//! The overlays are intentionally "`PoC` quality": they are based on loose
//! bounding boxes and anchor/handle positions in the model, and they do not
//! attempt to match the exact curve extents.

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
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for anchor in &shape.path.anchors {
        let screen = viewport.world_to_screen(anchor.pos);
        min_x = min_x.min(screen.x);
        min_y = min_y.min(screen.y);
        max_x = max_x.max(screen.x);
        max_y = max_y.max(screen.y);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some((Vec2::new(min_x, min_y), Vec2::new(max_x, max_y)))
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
mod tests {
    use super::*;
    use crate::model::{Anchor, PaintStyle, PathGeom, Rgba, SegmentKind, Shape};
    use rstest::rstest;
    use uuid::Uuid;

    fn shape_id(value: u128) -> ShapeId {
        ShapeId::from(Uuid::from_u128(value))
    }

    fn two_anchor_cubic_shape() -> Shape {
        Shape {
            id: shape_id(0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa),
            z: 0,
            style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 2.0, None),
            path: PathGeom {
                anchors: vec![
                    Anchor {
                        pos: Vec2::new(10.0, 20.0),
                        handle_in: None,
                        handle_out: Some(Vec2::new(20.0, 30.0)),
                    },
                    Anchor {
                        pos: Vec2::new(60.0, 80.0),
                        handle_in: Some(Vec2::new(50.0, 70.0)),
                        handle_out: None,
                    },
                ],
                segments: vec![SegmentKind::Cubic],
                closed: false,
                closing_segment: SegmentKind::Line,
            },
        }
    }

    #[rstest]
    fn selected_shape_overlays_include_anchor_and_handle_markers() {
        let shape = two_anchor_cubic_shape();
        let doc = Document {
            shapes: vec![shape.clone()],
        };
        let selection = Selection {
            items: vec![SelItem::Shape(shape.id)],
        };
        let viewport = Viewport::new();

        let overlays = compute_selection_overlays(&doc, &selection, viewport);

        assert_eq!(overlays.bbox_edges.len(), 4, "expected 4 bbox edges");
        assert_eq!(
            overlays.anchor_markers.len(),
            2,
            "expected markers for both anchors"
        );
        assert_eq!(
            overlays.handle_markers.len(),
            2,
            "expected markers for both handles"
        );
        assert_eq!(
            overlays.handle_lines.len(),
            2,
            "expected connector lines for both handles"
        );

        let first_anchor = shape.path.anchors.first().expect("anchor 0 exists");
        let second_anchor = shape.path.anchors.get(1).expect("anchor 1 exists");

        assert!(
            overlays
                .anchor_markers
                .iter()
                .any(|marker| marker.centre == first_anchor.pos)
        );
        assert!(
            overlays
                .anchor_markers
                .iter()
                .any(|marker| marker.centre == second_anchor.pos)
        );

        let handle_out = first_anchor.handle_out.expect("handle_out exists");
        let handle_in = second_anchor.handle_in.expect("handle_in exists");
        assert!(
            overlays
                .handle_markers
                .iter()
                .any(|marker| marker.centre == handle_out),
            "expected a handle marker at handle_out"
        );
        assert!(
            overlays
                .handle_markers
                .iter()
                .any(|marker| marker.centre == handle_in),
            "expected a handle marker at handle_in"
        );

        assert!(
            overlays
                .handle_lines
                .iter()
                .any(|line| line.start == first_anchor.pos && line.end == handle_out),
            "expected a connector line from anchor0 to handle_out"
        );
        assert!(
            overlays
                .handle_lines
                .iter()
                .any(|line| line.start == second_anchor.pos && line.end == handle_in),
            "expected a connector line from anchor1 to handle_in"
        );
    }

    #[rstest]
    fn non_shape_selection_does_not_infer_parent_shape_overlays() {
        let shape = two_anchor_cubic_shape();
        let doc = Document {
            shapes: vec![shape.clone()],
        };
        let selection = Selection {
            items: vec![SelItem::HandleOut {
                shape: shape.id,
                anchor: 0,
            }],
        };

        let overlays = compute_selection_overlays(&doc, &selection, Viewport::new());
        assert!(
            overlays.anchor_markers.is_empty()
                && overlays.handle_markers.is_empty()
                && overlays.handle_lines.is_empty()
                && overlays.bbox_edges.is_empty(),
            "expected no overlays when selection contains no Shape items"
        );
    }
}
