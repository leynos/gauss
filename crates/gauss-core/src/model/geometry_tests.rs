//! Unit tests for shape world bounds.
//!
//! These pin the segment and closing-edge traversal that
//! [`shape_world_bounds`](super::geometry::shape_world_bounds) performs.

use rstest::rstest;

use crate::model::{
    Anchor,
    PaintStyle,
    PathGeom,
    SegmentKind,
    Shape,
    ShapeId,
    Vec2,
    geometry::shape_world_bounds,
};

/// Build a shape from explicit path geometry with no styling distractions.
fn shape_with(path: PathGeom) -> Shape {
    Shape {
        id: ShapeId::from_accesskit_node_id(1),
        z: 0,
        style: PaintStyle::new(None, 1.0, None),
        path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Return bounds as `(min, max)` for direct comparison.
fn bounds_of(shape: &Shape) -> Option<(Vec2, Vec2)> {
    shape_world_bounds(shape).map(|bounds| (bounds.min(), bounds.max()))
}

#[rstest]
fn empty_path_has_no_bounds() {
    let shape = shape_with(PathGeom::new());

    assert_eq!(
        bounds_of(&shape),
        None,
        "a path with no anchors must produce no bounds"
    );
}

#[rstest]
fn anchor_positions_bound_a_line_path() {
    let mut path = PathGeom::new();
    path.anchors = vec![
        Anchor::new(Vec2::new(0.0, 0.0)),
        Anchor::new(Vec2::new(10.0, 5.0)),
    ];
    path.segments = vec![SegmentKind::Line];

    assert_eq!(
        bounds_of(&shape_with(path)),
        Some((Vec2::new(0.0, 0.0), Vec2::new(10.0, 5.0))),
        "line segments contribute only their anchors"
    );
}

#[rstest]
fn cubic_segment_extends_bounds_beyond_its_anchors() {
    let mut path = PathGeom::new();
    path.anchors = vec![
        Anchor {
            pos: Vec2::new(0.0, 0.0),
            handle_in: None,
            handle_out: Some(Vec2::new(0.0, 30.0)),
        },
        Anchor {
            pos: Vec2::new(10.0, 0.0),
            handle_in: Some(Vec2::new(10.0, 30.0)),
            handle_out: None,
        },
    ];
    path.segments = vec![SegmentKind::Cubic];

    let (min, max) = bounds_of(&shape_with(path)).expect("cubic path must have bounds");

    assert!(
        max.y > 20.0,
        "the curve must bulge above the anchors, got max.y = {}",
        max.y
    );
    assert_eq!(
        (min.x, max.x),
        (0.0, 10.0),
        "the curve must not exceed its anchor x range"
    );
}

#[rstest]
fn closed_path_without_a_cubic_closing_edge_stays_within_its_anchors() {
    let mut path = PathGeom::new();
    path.anchors = vec![
        Anchor::new(Vec2::new(0.0, 0.0)),
        Anchor::new(Vec2::new(10.0, 0.0)),
    ];
    path.segments = vec![SegmentKind::Line];
    path.closed = true;
    path.closing_segment = SegmentKind::Line;

    assert_eq!(
        bounds_of(&shape_with(path)),
        Some((Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0))),
        "a straight closing edge must add no extent of its own"
    );
}

#[rstest]
fn cubic_closing_edge_extends_bounds_using_last_out_and_first_in_handles() {
    let mut path = PathGeom::new();
    path.anchors = vec![
        Anchor {
            pos: Vec2::new(0.0, 0.0),
            handle_in: Some(Vec2::new(0.0, -40.0)),
            handle_out: None,
        },
        Anchor {
            pos: Vec2::new(10.0, 0.0),
            handle_in: None,
            handle_out: Some(Vec2::new(10.0, -40.0)),
        },
    ];
    path.segments = vec![SegmentKind::Line];
    path.closed = true;
    path.closing_segment = SegmentKind::Cubic;

    let (min, max) = bounds_of(&shape_with(path)).expect("closed path must have bounds");

    assert!(
        min.y < -20.0,
        "the cubic closing edge must bulge below the anchors, got min.y = {}",
        min.y
    );
    assert_eq!(
        max.y, 0.0,
        "the closing edge must not raise the maximum y beyond the anchors"
    );
}
