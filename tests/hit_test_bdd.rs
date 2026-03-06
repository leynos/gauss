//! Behaviour tests for the shared hit-test service.

use gauss::model::{
    Anchor, Document, HitTestIndex, Paint, PaintStyle, PathGeom, SegmentKind, SelectPointerHit,
    Shape, ShapeId, Vec2,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct HitTestWorld {
    document: Document,
    shape_ids: Vec<ShapeId>,
    cursor_world: Option<Vec2>,
    tolerance_world: Option<f32>,
    pointer_hit: Option<SelectPointerHit>,
    hover_hit: Option<SelectPointerHit>,
}

#[fixture]
fn world() -> HitTestWorld {
    HitTestWorld::default()
}

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

const fn sample_style() -> PaintStyle {
    PaintStyle {
        stroke: Paint::Solid(gauss::model::Rgba::new(16, 32, 64, 255)),
        stroke_width: 2.0,
        fill: Paint::None,
    }
}

fn square_shape_with_out_handle(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
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

fn index(world: &HitTestWorld) -> HitTestIndex<'_> {
    HitTestIndex::from_document(&world.document)
}

fn require_query_inputs(world: &HitTestWorld) -> TestSupportResult<(Vec2, f32)> {
    let cursor_world = world
        .cursor_world
        .ok_or_else(|| TestSupportError::missing("cursor_world", "hit-test query"))?;
    let tolerance_world = world
        .tolerance_world
        .ok_or_else(|| TestSupportError::missing("tolerance_world", "hit-test query"))?;
    Ok((cursor_world, tolerance_world))
}

#[given("a document with one square shape and an out handle")]
fn given_document_with_one_square(world: &mut HitTestWorld) {
    world.document = Document::new();
    world.shape_ids.clear();

    let id = shape_id(41);
    let _inserted = world.document.append_shape(square_shape_with_out_handle(
        id,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
    world.shape_ids.push(id);
}

#[given("a document with two overlapping square shapes")]
fn given_document_with_overlapping_shapes(world: &mut HitTestWorld) {
    world.document = Document::new();
    world.shape_ids.clear();

    let bottom = shape_id(51);
    let top = shape_id(52);

    let _bottom = world.document.append_shape(square_shape_with_out_handle(
        bottom,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
    let _top = world.document.append_shape(square_shape_with_out_handle(
        top,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));

    world.shape_ids.push(bottom);
    world.shape_ids.push(top);
}

#[given("the hit-test cursor is at world position {x:f32} {y:f32}")]
fn given_cursor(world: &mut HitTestWorld, x: f32, y: f32) {
    world.cursor_world = Some(Vec2::new(x, y));
}

#[given("the hit-test tolerance is {tolerance:f32}")]
fn given_tolerance(world: &mut HitTestWorld, tolerance: f32) {
    world.tolerance_world = Some(tolerance);
}

#[given("the hit-test tolerance is NaN")]
fn given_nan_tolerance(world: &mut HitTestWorld) {
    world.tolerance_world = Some(f32::NAN);
}

#[when("I run a pointer hit-test query")]
fn when_run_pointer_query(world: &mut HitTestWorld) -> TestSupportResult<()> {
    let (cursor_world, tolerance_world) = require_query_inputs(world)?;
    world.pointer_hit = Some(index(world).pointer_hit(cursor_world, tolerance_world));
    Ok(())
}

#[when("I run both pointer and hover hit-test queries")]
fn when_run_pointer_and_hover_query(world: &mut HitTestWorld) -> TestSupportResult<()> {
    let (cursor_world, tolerance_world) = require_query_inputs(world)?;
    let (pointer_hit, hover_hit) = {
        let hit_index = index(world);
        (
            hit_index.pointer_hit(cursor_world, tolerance_world),
            hit_index.hover_hit(cursor_world, tolerance_world),
        )
    };
    world.pointer_hit = Some(pointer_hit);
    world.hover_hit = Some(hover_hit);
    Ok(())
}

fn require_pointer_hit(world: &HitTestWorld) -> TestSupportResult<SelectPointerHit> {
    world
        .pointer_hit
        .ok_or_else(|| TestSupportError::missing("pointer_hit", "assertion"))
}

#[derive(Clone, Copy, Debug)]
enum HitExpectation {
    Handle { anchor_index: usize },
    Segment { seg_index: usize },
    TopmostShape { shape_index: usize },
}

impl HitExpectation {
    fn selector(self) -> fn(&[ShapeId]) -> Option<&ShapeId> {
        match self {
            Self::TopmostShape { .. } => <[ShapeId]>::last,
            Self::Handle { .. } | Self::Segment { .. } => <[ShapeId]>::first,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Handle { .. } => "handle assertion",
            Self::Segment { .. } => "segment assertion",
            Self::TopmostShape { .. } => "topmost assertion",
        }
    }

    fn matches_hit(self, hit: SelectPointerHit, expected_shape: ShapeId) -> bool {
        match self {
            Self::Handle { anchor_index } => matches!(
                hit,
                SelectPointerHit::Handle(handle_hit)
                    if handle_hit.shape_id == expected_shape && handle_hit.anchor_index == anchor_index
            ),
            Self::Segment { seg_index } => matches!(
                hit,
                SelectPointerHit::Segment(segment_hit)
                    if segment_hit.shape_id == expected_shape && segment_hit.seg_index == seg_index
            ),
            Self::TopmostShape { shape_index } => matches!(
                hit,
                SelectPointerHit::Shape(shape_hit)
                    if shape_hit.shape_id == expected_shape && shape_hit.shape_index == shape_index
            ),
        }
    }
}

fn assert_pointer_hit_matches(
    world: &HitTestWorld,
    expected: HitExpectation,
) -> TestSupportResult<()> {
    let expected_shape = expected.selector()(&world.shape_ids)
        .copied()
        .ok_or_else(|| TestSupportError::missing("shape_id", expected.label()))?;
    let hit = require_pointer_hit(world)?;
    if !expected.matches_hit(hit, expected_shape) {
        return Err(TestSupportError::expectation(format!(
            "expected {expected:?} for {expected_shape:?}; got {hit:?}"
        )));
    }
    Ok(())
}

#[then("the hit-test result is Handle")]
fn then_result_handle(world: &HitTestWorld) -> TestSupportResult<()> {
    assert_pointer_hit_matches(world, HitExpectation::Handle { anchor_index: 0 })
}

#[then("the hit-test result is Segment")]
fn then_result_segment(world: &HitTestWorld) -> TestSupportResult<()> {
    assert_pointer_hit_matches(world, HitExpectation::Segment { seg_index: 0 })
}

#[then("the hit-test result is TopmostShape")]
fn then_result_topmost_shape(world: &HitTestWorld) -> TestSupportResult<()> {
    assert_pointer_hit_matches(world, HitExpectation::TopmostShape { shape_index: 1 })
}

#[then("the hit-test result is None")]
fn then_result_none(world: &HitTestWorld) -> TestSupportResult<()> {
    let hit = require_pointer_hit(world)?;
    if hit != SelectPointerHit::None {
        return Err(TestSupportError::expectation(format!(
            "expected no hit; got {hit:?}"
        )));
    }
    Ok(())
}

#[then("the pointer and hover hit-test results match")]
fn then_pointer_and_hover_match(world: &HitTestWorld) -> TestSupportResult<()> {
    let pointer_hit = world
        .pointer_hit
        .ok_or_else(|| TestSupportError::missing("pointer_hit", "match assertion"))?;
    let hover_hit = world
        .hover_hit
        .ok_or_else(|| TestSupportError::missing("hover_hit", "match assertion"))?;
    if pointer_hit != hover_hit {
        return Err(TestSupportError::expectation(format!(
            "expected pointer and hover hits to match; pointer={pointer_hit:?} hover={hover_hit:?}"
        )));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/hit_test.feature",
    name = "Handle hits have highest priority"
)]
fn handle_hits_have_highest_priority(world: HitTestWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/hit_test.feature",
    name = "Segment hits win over shape-body fallback"
)]
fn segment_hits_win_over_shape_fallback(world: HitTestWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/hit_test.feature",
    name = "Hover and pointer queries match"
)]
fn hover_and_pointer_queries_match(world: HitTestWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/hit_test.feature",
    name = "Overlapping shapes resolve to the topmost shape"
)]
fn overlapping_shapes_resolve_to_topmost(world: HitTestWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/hit_test.feature",
    name = "Non-finite tolerance produces no hit"
)]
fn non_finite_tolerance_produces_no_hit(world: HitTestWorld) {
    let _ = world;
}
