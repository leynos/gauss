//! Behaviour tests for Command dispatch.
//!
//! These tests use `rstest-bdd` to validate that commands correctly bridge
//! Actions to undoable document mutations.

use gauss::model::{
    Action, Anchor, Command, CommandError, CommandInverse, Document, PaintStyle, PathGeom, Rgba,
    SegmentKind, SelItem, Selection, Shape, ShapeId, Vec2, prepare_command,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};
use uuid::Uuid;

/// World state for command BDD tests.
#[derive(Default)]
struct CommandWorld {
    doc: Document,
    selection: Selection,
    command: Option<Result<Command, CommandError>>,
    inverse: Option<CommandInverse>,
}

#[fixture]
fn world() -> CommandWorld {
    CommandWorld::default()
}

// === Test helpers ===

#[must_use]
fn shape_id(seed: u128) -> ShapeId {
    ShapeId::from(Uuid::from_u128(seed))
}

#[must_use]
fn sample_shape(id: ShapeId, z: i32) -> Shape {
    let mut path = PathGeom::new();
    path.anchors.push(Anchor::new(Vec2::new(10.0, 20.0)));
    path.anchors.push(Anchor {
        pos: Vec2::new(30.0, 40.0),
        handle_in: Some(Vec2::new(25.0, 35.0)),
        handle_out: None,
    });
    path.segments.push(SegmentKind::Line);

    Shape {
        id,
        z,
        style: PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), 2.0, None),
        path,
    }
}

// === Given steps ===

#[given("a document with two shapes")]
fn given_doc_with_two_shapes(world: &mut CommandWorld) {
    world.doc = Document {
        shapes: vec![sample_shape(shape_id(1), 0), sample_shape(shape_id(2), 1)],
    };
}

#[given("the first shape is selected")]
fn given_first_shape_selected(world: &mut CommandWorld) {
    world.selection.toggle(SelItem::Shape(shape_id(1)));
}

#[given("nothing is selected")]
fn given_nothing_selected(world: &mut CommandWorld) {
    world.selection = Selection::default();
}

// === When steps ===

#[when("I prepare DeleteSelection action")]
fn when_prepare_delete_selection(world: &mut CommandWorld) {
    world.command = Some(prepare_command(
        Action::DeleteSelection,
        &world.doc,
        &world.selection,
    ));
}

#[when("I apply the command")]
fn when_apply_command(world: &mut CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "apply"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    let inverse = cmd
        .apply(&mut world.doc)
        .map_err(|e| TestSupportError::expectation(format!("apply failed: {e}")))?;

    world.inverse = Some(inverse);
    Ok(())
}

#[when("I apply the inverse")]
fn when_apply_inverse(world: &mut CommandWorld) -> TestSupportResult<()> {
    let inverse = world
        .inverse
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("inverse", "undo"))?;

    inverse
        .apply(&mut world.doc)
        .map_err(|e| TestSupportError::expectation(format!("undo failed: {e}")))?;

    Ok(())
}

// === Then steps ===

#[then("the command should be DeleteShapes")]
fn then_command_is_delete_shapes(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = get_command(world)?;
    match cmd {
        Command::DeleteShapes { .. } => Ok(()),
        _ => Err(TestSupportError::expectation("expected DeleteShapes")),
    }
}

#[then("the command should target one shape")]
fn then_command_targets_one_shape(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = get_command(world)?;
    match cmd {
        Command::DeleteShapes { targets } if targets.len() == 1 => Ok(()),
        Command::DeleteShapes { targets } => Err(TestSupportError::expectation(format!(
            "expected 1 target, got {}",
            targets.len()
        ))),
        _ => Err(TestSupportError::expectation("expected DeleteShapes")),
    }
}

#[then("the document should have one shape")]
fn then_doc_has_one_shape(world: &CommandWorld) -> TestSupportResult<()> {
    if world.doc.shapes.len() == 1 {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected 1 shape, got {}",
            world.doc.shapes.len()
        )))
    }
}

#[then("the document should have two shapes")]
fn then_doc_has_two_shapes(world: &CommandWorld) -> TestSupportResult<()> {
    if world.doc.shapes.len() == 2 {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected 2 shapes, got {}",
            world.doc.shapes.len()
        )))
    }
}

#[then("the command should fail with EmptySelection")]
fn then_command_fails_empty_selection(world: &CommandWorld) -> TestSupportResult<()> {
    match &world.command {
        Some(Err(CommandError::EmptySelection)) => Ok(()),
        Some(Err(e)) => Err(TestSupportError::expectation(format!(
            "expected EmptySelection, got {e}"
        ))),
        Some(Ok(_)) => Err(TestSupportError::expectation(
            "expected error, got success".to_owned(),
        )),
        None => Err(TestSupportError::missing("command", "check")),
    }
}

#[then(r#"the command name should be "Delete""#)]
fn then_command_name_is_delete(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = get_command(world)?;
    if cmd.name() == "Delete" {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            r#"expected "Delete", got "{}""#,
            cmd.name()
        )))
    }
}

// === Helper functions ===

fn get_command(world: &CommandWorld) -> TestSupportResult<&Command> {
    world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))
}

// === Scenario bindings ===

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection produces valid command"
)]
fn delete_selection_produces_valid_command(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection command removes shape"
)]
fn delete_selection_command_removes_shape(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection is undoable"
)]
fn delete_selection_is_undoable(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection requires selection"
)]
fn delete_selection_requires_selection(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Command has human-readable name"
)]
fn command_has_human_readable_name(world: CommandWorld) {
    let _ = world;
}
