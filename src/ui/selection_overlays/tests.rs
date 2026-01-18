//! Tests for selection overlay geometry helpers.

use super::*;
use crate::model::{
    Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, SelItem, Selection, Shape, ShapeId,
    Vec2, Viewport,
};
use rstest::rstest;

const TEST_ID_VERSION: u32 = 0xffff_fffe;

fn shape_id(seed: u32) -> ShapeId {
    let raw = (u64::from(TEST_ID_VERSION) << 32) | u64::from(seed);
    ShapeId::from_accesskit_node_id(raw)
}

fn two_anchor_cubic_shape() -> Shape {
    Shape {
        id: shape_id(0xaaaa_aaaa),
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

fn bbox_from_edges(edges: &[OverlayLine]) -> Option<(Vec2, Vec2)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for edge in edges {
        min_x = min_x.min(edge.start.x).min(edge.end.x);
        min_y = min_y.min(edge.start.y).min(edge.end.y);
        max_x = max_x.max(edge.start.x).max(edge.end.x);
        max_y = max_y.max(edge.start.y).max(edge.end.y);
    }

    if !bounds_are_finite(min_x, min_y, max_x, max_y) {
        return None;
    }

    Some((Vec2::new(min_x, min_y), Vec2::new(max_x, max_y)))
}

fn bounds_are_finite(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> bool {
    [min_x, min_y, max_x, max_y].into_iter().all(f32::is_finite)
}

#[rstest]
fn selected_shape_overlays_include_anchor_and_handle_markers() {
    let shape = two_anchor_cubic_shape();
    let mut doc = Document::new();
    doc.append_shape(shape.clone());
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
    let mut doc = Document::new();
    doc.append_shape(shape.clone());
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

#[rstest]
fn bbox_accounts_for_cubic_curve_extents() {
    let shape = Shape {
        id: shape_id(0xbbbb_bbbb),
        z: 0,
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 2.0, None),
        path: PathGeom {
            anchors: vec![
                Anchor {
                    pos: Vec2::new(0.0, 0.0),
                    handle_in: None,
                    handle_out: Some(Vec2::new(0.0, 10.0)),
                },
                Anchor {
                    pos: Vec2::new(10.0, 0.0),
                    handle_in: Some(Vec2::new(10.0, 10.0)),
                    handle_out: None,
                },
            ],
            segments: vec![SegmentKind::Cubic],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
    };

    let mut doc = Document::new();
    doc.append_shape(shape.clone());
    let selection = Selection {
        items: vec![SelItem::Shape(shape.id)],
    };

    let overlays = compute_selection_overlays(&doc, &selection, Viewport::new());
    let (_min, max) = bbox_from_edges(&overlays.bbox_edges).expect("bbox edges should exist");

    assert!(
        max.y > 6.0,
        "expected bbox to include the cubic bulge, got max y {}",
        max.y
    );
}
