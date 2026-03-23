//! Behaviour tests for Gauss model operations.
//!
//! These tests use `rstest-bdd` to validate user-facing behaviour at the
//! controller/model boundary, without a running GPUI window.

use gauss_core::model::{
    Anchor, DocChange, DocOp, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId,
    Vec2,
};
use gauss_svg::svg::export::{CanvasSize, export_svg};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct DocWorld {
    doc: Document,
    last_change: Option<DocChange>,
    last_svg: Option<String>,
}

#[fixture]
fn world() -> DocWorld {
    DocWorld::default()
}

#[given("an empty document")]
fn empty_document(world: &mut DocWorld) {
    world.doc = Document::new();
    world.last_change = None;
    world.last_svg = None;
}

#[when("I insert a shape")]
fn insert_shape(world: &mut DocWorld) {
    let shape = Shape {
        id: ShapeId::default(),
        z: 0,
        style: PaintStyle::new(None, 1.0, None),
        path: PathGeom::new(),
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    };
    let change = DocChange {
        ops: vec![DocOp::InsertShape { index: 0, shape }],
    };

    change.apply(&mut world.doc);
    world.last_change = Some(change);
}

#[when("I undo the insertion")]
fn undo_insertion(world: &mut DocWorld) -> TestSupportResult<()> {
    let change = world
        .last_change
        .clone()
        .ok_or_else(|| TestSupportError::missing("doc change", "undo insertion"))?;
    change.apply_inverse(&mut world.doc);
    Ok(())
}

#[when("I add a closed triangle path")]
fn add_closed_triangle(world: &mut DocWorld) {
    let shape = Shape {
        id: ShapeId::default(),
        z: 0,
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(1.0, 1.0)),
                Anchor::new(Vec2::new(2.0, 1.0)),
                Anchor::new(Vec2::new(2.0, 2.0)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    };
    world.doc.append_shape(shape);
}

#[when("I export the document as an SVG")]
fn export_document_as_svg(world: &mut DocWorld) {
    let svg = export_svg(&world.doc, CanvasSize::new(100.0, 100.0));
    world.last_svg = Some(svg);
}

#[then("the document contains {count:usize} shapes")]
fn document_contains_shapes(world: &DocWorld, count: usize) -> TestSupportResult<()> {
    if world.doc.len() != count {
        return Err(TestSupportError::expectation(format!(
            "expected {count} shapes, got {}",
            world.doc.len()
        )));
    }
    Ok(())
}

#[then("the exported SVG contains {snippet}")]
fn exported_svg_contains(world: &DocWorld, snippet: String) -> TestSupportResult<()> {
    let svg = world
        .last_svg
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("exported svg", "after export"))?;

    if !svg.contains(&snippet) {
        return Err(TestSupportError::expectation(format!(
            "Expected exported SVG to contain snippet.\nSnippet: {snippet}\nSVG: {svg}"
        )));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/gauss_model_ops.feature",
    name = "Insert and undo a shape"
)]
fn insert_and_undo_shape(world: DocWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/gauss_model_ops.feature",
    name = "Export a closed triangle path"
)]
fn export_closed_triangle_contains_close_command(world: DocWorld) {
    let _ = world;
}
