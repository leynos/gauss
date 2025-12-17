//! Behaviour tests for Gauss model operations.
//!
//! These tests use `rstest-bdd` to validate user-facing behaviour at the
//! controller/model boundary, without a running GPUI window.

use gauss::model::{DocChange, DocOp, Document, PaintStyle, PathGeom, Shape, ShapeId};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};

#[derive(Default)]
struct DocWorld {
    doc: Document,
    last_change: Option<DocChange>,
}

#[fixture]
fn world() -> DocWorld {
    DocWorld::default()
}

#[given("an empty document")]
fn empty_document(world: &mut DocWorld) {
    world.doc = Document::new();
    world.last_change = None;
}

#[when("I insert a shape")]
fn insert_shape(world: &mut DocWorld) {
    let shape = Shape {
        id: ShapeId::new_v4(),
        z: 0,
        style: PaintStyle::new(None, 1.0, None),
        path: PathGeom::new(),
    };
    let change = DocChange {
        ops: vec![DocOp::InsertShape { index: 0, shape }],
    };

    change.apply(&mut world.doc);
    world.last_change = Some(change);
}

#[when("I undo the insertion")]
fn undo_insertion(world: &mut DocWorld) {
    let Some(change) = world.last_change.clone() else {
        panic!("Expected a previous change to undo");
    };
    change.apply_inverse(&mut world.doc);
}

#[then("the document contains {count:usize} shapes")]
fn document_contains_shapes(world: &DocWorld, count: usize) {
    assert_eq!(world.doc.shapes.len(), count);
}

#[scenario(
    path = "tests/features/gauss_model_ops.feature",
    name = "Insert and undo a shape"
)]
fn insert_and_undo_shape(world: DocWorld) {
    let _ = world;
}
