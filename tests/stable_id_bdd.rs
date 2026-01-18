//! Behaviour tests for stable shape identifiers.
//!
//! These tests use `rstest-bdd` to validate that shape IDs remain stable when
//! documents are reordered and when IDs are mapped to AccessKit node IDs.

use gauss::model::{Document, PaintStyle, PathGeom, Shape, ShapeId};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct IdWorld {
    doc: Document,
    original_ids: Vec<ShapeId>,
    round_tripped: Option<ShapeId>,
}

#[fixture]
fn world() -> IdWorld {
    IdWorld::default()
}

fn append_shapes(world: &mut IdWorld, count: usize, clear_existing: bool) {
    if clear_existing {
        world.original_ids.clear();
    }

    for _ in 0..count {
        let shape = Shape {
            id: ShapeId::default(),
            z: 0,
            style: PaintStyle::new(None, 1.0, None),
            path: PathGeom::new(),
        };
        let id = world.doc.append_shape(shape);
        world.original_ids.push(id);
    }
}

#[given("an empty document")]
fn empty_document(world: &mut IdWorld) {
    world.doc = Document::new();
    world.original_ids.clear();
    world.round_tripped = None;
}

#[when("I append two shapes")]
fn append_two_shapes(world: &mut IdWorld) {
    append_shapes(world, 2, false);
}

#[when("I reorder the first shape to the end")]
fn reorder_first_shape(world: &mut IdWorld) -> TestSupportResult<()> {
    if !world.doc.reorder(0, 1) {
        return Err(TestSupportError::expectation(
            "expected reorder to succeed for two shapes",
        ));
    }
    Ok(())
}

#[then("the document still contains the same shape IDs")]
fn document_contains_same_ids(world: &IdWorld) -> TestSupportResult<()> {
    let mut expected = world.original_ids.clone();
    expected.sort_by_key(|id| id.to_accesskit_node_id());

    let mut actual: Vec<_> = world.doc.iter_ids_in_draw_order().collect();
    actual.sort_by_key(|id| id.to_accesskit_node_id());

    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected IDs to remain stable, expected {expected:?}, got {actual:?}"
        )));
    }

    Ok(())
}

#[when("I append a shape")]
fn append_shape(world: &mut IdWorld) {
    world.original_ids.clear();
    append_shapes(world, 1, false);
}

#[when("I convert the shape id to an AccessKit node id")]
fn convert_shape_id(world: &mut IdWorld) -> TestSupportResult<()> {
    let id = world
        .original_ids
        .first()
        .copied()
        .ok_or_else(|| TestSupportError::missing("shape id", "AccessKit conversion"))?;
    let round_tripped = ShapeId::from_accesskit_node_id(id.to_accesskit_node_id());
    world.round_tripped = Some(round_tripped);
    Ok(())
}

#[then("the reconstructed shape id matches the original")]
fn reconstructed_id_matches(world: &IdWorld) -> TestSupportResult<()> {
    let expected = world
        .original_ids
        .first()
        .copied()
        .ok_or_else(|| TestSupportError::missing("shape id", "comparison"))?;
    let actual = world
        .round_tripped
        .ok_or_else(|| TestSupportError::missing("round-tripped id", "comparison"))?;

    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected round-trip ID to match {expected:?}, got {actual:?}"
        )));
    }

    Ok(())
}

#[scenario(
    path = "tests/features/stable_ids.feature",
    name = "Shape IDs remain stable when reordered"
)]
fn shape_ids_remain_stable_when_reordered(world: IdWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/stable_ids.feature",
    name = "AccessKit node IDs round trip"
)]
fn accesskit_node_ids_round_trip(world: IdWorld) {
    let _ = world;
}
