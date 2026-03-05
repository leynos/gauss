//! Unit tests for shared hit-testing.

use super::hit_test::{HitTestBackend, HitTestIndex};
use super::{
    Anchor, Document, Paint, PaintStyle, PathGeom, SegmentKind, SelItem, SelectPointerHit,
    Selection, Shape, ShapeId, Vec2,
};
use rstest::rstest;

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

const fn sample_style() -> PaintStyle {
    PaintStyle {
        stroke: Paint::Solid(super::Rgba::new(16, 32, 64, 255)),
        stroke_width: 2.0,
        fill: Paint::None,
    }
}

fn square_shape(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
    let mut first_anchor = Anchor::new(min);
    first_anchor.handle_out = Some(min.add(Vec2::new(2.0, 0.0)));

    Shape {
        id,
        z: 0,
        style: sample_style(),
        path: PathGeom {
            anchors: vec![
                first_anchor,
                Anchor::new(Vec2::new(max.x, min.y)),
                Anchor::new(max),
                Anchor::new(Vec2::new(min.x, max.y)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

fn single_square_document() -> (Document, ShapeId) {
    let mut document = Document::new();
    let id = shape_id(7);
    let _inserted =
        document.append_shape(square_shape(id, Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)));

    (document, id)
}

#[rstest]
fn reports_linear_scan_backend_by_default() {
    let (document, _shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    assert_eq!(index.backend(), HitTestBackend::LinearScan);
}

#[rstest]
fn resolves_handle_before_anchor_segment_and_shape() {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(2.0, 0.0), 0.5);

    assert!(matches!(
        hit,
        SelectPointerHit::Handle(handle_hit)
            if handle_hit.shape_id == shape_id
                && handle_hit.anchor_index == 0
    ));
}

#[rstest]
fn resolves_anchor_before_segment_and_shape() {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(0.0, 0.0), 0.5);

    assert!(matches!(
        hit,
        SelectPointerHit::Anchor(anchor_hit)
            if anchor_hit.shape_id == shape_id
                && anchor_hit.anchor_index == 0
    ));
}

#[rstest]
fn resolves_segment_before_shape() {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(5.0, 0.0), 0.5);

    assert!(matches!(
        hit,
        SelectPointerHit::Segment(segment_hit)
            if segment_hit.shape_id == shape_id
                && segment_hit.seg_index == 0
    ));
}

#[rstest]
fn resolves_shape_as_final_fallback() {
    let (document, shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(5.0, 5.0), 0.5);

    assert!(matches!(
        hit,
        SelectPointerHit::Shape(shape_hit)
            if shape_hit.shape_id == shape_id
    ));
}

#[rstest]
fn returns_none_when_outside_all_targets() {
    let (document, _shape_id) = single_square_document();
    let index = HitTestIndex::from_document(&document);
    let hit = index.pointer_hit(Vec2::new(20.0, 20.0), 0.5);

    assert_eq!(hit, SelectPointerHit::None);
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
