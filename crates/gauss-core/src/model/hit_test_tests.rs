//! Unit tests for shared hit-testing.

use super::hit_test::{HitTestBackend, HitTestIndex};
use super::{Document, SelItem, SelectHandleHitKind, SelectPointerHit, Selection, ShapeId, Vec2};
use crate::test_helpers::{
    cubic_shape, handle_in_only_shape, square_shape, square_shape_with_out_handle,
};
use rstest::rstest;

#[derive(Debug)]
enum ExpectedHit {
    Handle {
        anchor_index: usize,
        kind: SelectHandleHitKind,
    },
    Anchor {
        anchor_index: usize,
    },
    Segment {
        seg_index: usize,
    },
    Shape,
    None,
}

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn is_expected_handle_hit(
    hit: &SelectPointerHit,
    shape_id: ShapeId,
    anchor_index: usize,
    kind: SelectHandleHitKind,
) -> bool {
    let SelectPointerHit::Handle(handle_hit) = hit else {
        return false;
    };

    (
        handle_hit.shape_id,
        handle_hit.anchor_index,
        handle_hit.kind,
    ) == (shape_id, anchor_index, kind)
}

fn single_square_document() -> (Document, ShapeId) {
    let mut document = Document::new();
    let id = shape_id(7);
    let _inserted = document.append_shape(square_shape_with_out_handle(
        id,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(2.0, 0.0),
    ));

    (document, id)
}

#[rstest]
fn reports_linear_scan_backend_by_default() {
    let (document, _shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    assert_eq!(index.backend(), HitTestBackend::LinearScan);
}

#[rstest]
#[case(
    "handle beats anchor/segment/shape",
    Vec2::new(2.0, 0.0),
    0.5,
    ExpectedHit::Handle {
        anchor_index: 0,
        kind: SelectHandleHitKind::Out,
    }
)]
#[case(
    "anchor beats segment/shape",
    Vec2::new(0.0, 0.0),
    0.5,
    ExpectedHit::Anchor { anchor_index: 0 }
)]
#[case(
    "segment beats shape",
    Vec2::new(5.0, 0.0),
    0.5,
    ExpectedHit::Segment { seg_index: 0 }
)]
#[case(
    "shape as final fallback",
    Vec2::new(5.0, 5.0),
    0.5,
    ExpectedHit::Shape
)]
#[case(
    "none when outside all targets",
    Vec2::new(20.0, 20.0),
    0.5,
    ExpectedHit::None
)]
fn priority_resolution(
    #[case] label: &str,
    #[case] point: Vec2,
    #[case] tolerance: f32,
    #[case] expected: ExpectedHit,
) {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(point, tolerance);

    match expected {
        ExpectedHit::Handle { anchor_index, kind } => assert!(
            is_expected_handle_hit(&hit, shape_id, anchor_index, kind),
            "{label}: expected Handle(anchor_index={anchor_index}, kind={kind:?}); got {hit:?}"
        ),
        ExpectedHit::Anchor { anchor_index } => assert!(
            matches!(&hit, SelectPointerHit::Anchor(anchor_hit)
                if anchor_hit.shape_id == shape_id && anchor_hit.anchor_index == anchor_index),
            "{label}: expected Anchor(anchor_index={anchor_index}); got {hit:?}"
        ),
        ExpectedHit::Segment { seg_index } => assert!(
            matches!(&hit, SelectPointerHit::Segment(segment_hit)
                if segment_hit.shape_id == shape_id && segment_hit.seg_index == seg_index),
            "{label}: expected Segment(seg_index={seg_index}); got {hit:?}"
        ),
        ExpectedHit::Shape => assert!(
            matches!(&hit, SelectPointerHit::Shape(shape_hit) if shape_hit.shape_id == shape_id),
            "{label}: expected Shape; got {hit:?}"
        ),
        ExpectedHit::None => assert_eq!(hit, SelectPointerHit::None, "{label}"),
    }
}

#[rstest]
fn handle_in_pointer_hit_returns_handle_in() {
    let mut document = Document::new();
    let shape_id = shape_id(21);
    let anchor_pos = Vec2::new(100.0, 100.0);
    let _inserted = document.append_shape(handle_in_only_shape(
        shape_id,
        anchor_pos,
        Vec2::new(-4.0, 0.0),
    ));

    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(anchor_pos.add(Vec2::new(-4.0, 0.0)), 0.5);

    assert!(is_expected_handle_hit(
        &hit,
        shape_id,
        0,
        SelectHandleHitKind::In,
    ));
}

#[rstest]
fn cubic_segment_pointer_hit_returns_segment() {
    let mut document = Document::new();
    let shape_id = shape_id(22);
    let start = Vec2::new(50.0, 100.0);
    let end = Vec2::new(150.0, 100.0);
    let _inserted = document.append_shape(cubic_shape(
        shape_id,
        start,
        end,
        Vec2::new(33.333_332, -20.0),
    ));

    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(100.0, 100.0), 2.0);

    assert!(matches!(
        hit,
        SelectPointerHit::Segment(segment_hit)
            if segment_hit.shape_id == shape_id && segment_hit.seg_index == 0
    ));
}

#[rstest]
fn selects_topmost_shape_when_bounding_boxes_overlap() {
    let mut document = Document::new();
    let bottom = shape_id(11);
    let top = shape_id(12);
    let _bottom = document.append_shape(square_shape(
        bottom,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
    let _top = document.append_shape(square_shape(
        top,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));

    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(5.0, 5.0), 0.5);

    assert!(matches!(
        hit,
        SelectPointerHit::Shape(shape_hit)
            if shape_hit.shape_id == top && shape_hit.shape_index == 1
    ));
}

#[rstest]
fn treats_negative_tolerance_as_zero() {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(5.0, 0.0), -1.0);

    assert!(matches!(
        hit,
        SelectPointerHit::Segment(segment_hit)
            if segment_hit.shape_id == shape_id
    ));
}

#[rstest]
#[case(f32::NAN)]
#[case(f32::INFINITY)]
#[case(f32::NEG_INFINITY)]
fn returns_none_for_non_finite_tolerance(#[case] tolerance: f32) {
    let (document, _shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(5.0, 5.0), tolerance);

    assert_eq!(hit, SelectPointerHit::None);
}

#[rstest]
fn hover_and_pointer_queries_use_identical_resolution() {
    let (document, _shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let pointer_hit = index.pointer_hit(Vec2::new(5.0, 0.0), 0.5);
    let hover_hit = index.hover_hit(Vec2::new(5.0, 0.0), 0.5);

    assert_eq!(pointer_hit, hover_hit);
}

#[rstest]
fn segment_hit_can_be_toggled_into_selection() {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(5.0, 0.0), 0.5);

    let mut selection = Selection::empty();
    if let SelectPointerHit::Segment(segment_hit) = hit {
        selection.items.push(SelItem::Shape(shape_id));
        selection.items.push(SelItem::Segment {
            shape: segment_hit.shape_id,
            seg: segment_hit.seg_index,
        });
    }

    assert!(selection.contains(&SelItem::Shape(shape_id)));
    assert!(selection.items.iter().any(
        |item| matches!(item, SelItem::Segment { shape, seg } if *shape == shape_id && *seg == 0)
    ));
}
