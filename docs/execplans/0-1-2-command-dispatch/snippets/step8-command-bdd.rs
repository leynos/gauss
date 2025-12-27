//! Behaviour tests for Command dispatch.

use gauss::model::{
    Action, Command, CommandInverse, Document, Selection, SelItem,
    Shape, UserError, prepare_command,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

/// World state for command BDD tests.
#[derive(Default)]
struct CommandWorld {
    doc: Document,
    selection: Selection,
    command: Option<Result<Command, UserError>>,
    inverse: Option<CommandInverse>,
}

#[fixture]
fn world() -> CommandWorld {
    CommandWorld::default()
}

// === Given steps ===

#[given("a document with two shapes")]
fn given_doc_with_two_shapes(world: &mut CommandWorld) {
    world.doc = Document::default();
    world.doc.shapes.push(Shape::default());
    world.doc.shapes.push(Shape::default());
}

#[given("the first shape is selected")]
fn given_first_shape_selected(world: &mut CommandWorld) {
    if let Some(shape) = world.doc.shapes.first() {
        world.selection.toggle(SelItem::Shape(shape.id));
    }
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
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    match cmd {
        Command::DeleteShapes { .. } => Ok(()),
        #[expect(
            unreachable_patterns,
            reason = "Command is currently single-variant; arm guards future variants"
        )]
        _ => Err(TestSupportError::expectation("expected DeleteShapes")),
    }
}

#[then("the command should target one shape")]
fn then_command_targets_one_shape(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    match cmd {
        Command::DeleteShapes { targets } if targets.len() == 1 => Ok(()),
        Command::DeleteShapes { targets } => Err(TestSupportError::expectation(format!(
            "expected 1 target, got {}",
            targets.len()
        ))),
        #[expect(
            unreachable_patterns,
            reason = "Command is currently single-variant; arm guards future variants"
        )]
        _ => Err(TestSupportError::expectation("expected DeleteShapes")),
    }
}

/// Helper: assert document has expected number of shapes.
fn assert_doc_shape_count(world: &CommandWorld, expected: usize) -> TestSupportResult<()> {
    if world.doc.shapes.len() == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected {} shape{}, got {}",
            expected,
            if expected == 1 { "" } else { "s" },
            world.doc.shapes.len()
        )))
    }
}

#[then("the document should have one shape")]
fn then_doc_has_one_shape(world: &CommandWorld) -> TestSupportResult<()> {
    assert_doc_shape_count(world, 1)
}

#[then("the document should have two shapes")]
fn then_doc_has_two_shapes(world: &CommandWorld) -> TestSupportResult<()> {
    assert_doc_shape_count(world, 2)
}

#[then("the command should fail with EmptySelection")]
fn then_command_fails_empty_selection(world: &CommandWorld) -> TestSupportResult<()> {
    match &world.command {
        Some(Err(UserError::EmptySelection)) => Ok(()),
        Some(Err(e)) => Err(TestSupportError::expectation(format!(
            "expected EmptySelection, got {e}"
        ))),
        Some(Ok(_)) => Err(TestSupportError::expectation(
            "expected error, got success",
        )),
        None => Err(TestSupportError::missing("command", "check")),
    }
}

#[then(r#"the command name should be "Delete""#)]
fn then_command_name_is_delete(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    if cmd.name() == "Delete" {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            r#"expected "Delete", got "{}""#,
            cmd.name()
        )))
    }
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
