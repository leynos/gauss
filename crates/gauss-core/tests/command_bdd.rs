//! Behaviour tests for Command dispatch.
//!
//! These tests use `rstest-bdd` to validate that commands correctly bridge
//! Actions to undoable document mutations.

use gauss_core::model::{
    Action, Command, CommandInverse, Document, EngineState, HistoryError, SelItem, Selection,
    UserError, prepare_command,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::shapes::{sample_shape, shape_id};
use test_support::{TestSupportError, TestSupportResult};

/// World state for command BDD tests.
#[derive(Default)]
struct CommandWorld {
    state: EngineState,
    command: Option<Result<Command, UserError>>,
    inverse: Option<CommandInverse>,
    history_error: Option<HistoryError>,
}

#[fixture]
fn world() -> CommandWorld {
    CommandWorld::default()
}

// === Given steps ===

#[given("a document with two shapes")]
fn given_doc_with_two_shapes(world: &mut CommandWorld) {
    let mut doc = Document::new();
    doc.append_shape(sample_shape(shape_id(1), 0));
    doc.append_shape(sample_shape(shape_id(2), 1));
    world.state.document = doc;
}

#[given("the first shape is selected")]
fn given_first_shape_selected(world: &mut CommandWorld) {
    world.state.selection.toggle(SelItem::Shape(shape_id(1)));
}

#[given("nothing is selected")]
fn given_nothing_selected(world: &mut CommandWorld) {
    world.state.selection = Selection::default();
}

// === When steps ===

#[when("I prepare DeleteSelection action")]
fn when_prepare_delete_selection(world: &mut CommandWorld) {
    world.command = Some(prepare_command(Action::DeleteSelection, &world.state));
}

#[when("I prepare RaiseSelection action")]
fn when_prepare_raise_selection(world: &mut CommandWorld) {
    world.command = Some(prepare_command(Action::RaiseSelection, &world.state));
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
        .apply(&mut world.state.document)
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
        .apply(&mut world.state.document)
        .map_err(|e| TestSupportError::expectation(format!("undo failed: {e}")))?;

    Ok(())
}

#[when("I undo on an empty history")]
fn when_undo_on_empty_history(world: &mut CommandWorld) -> TestSupportResult<()> {
    world
        .state
        .undo_document()
        .map_err(|e| TestSupportError::expectation(format!("empty undo failed: {e}")))?;
    Ok(())
}

#[when("I apply DeleteSelection through EngineState history")]
fn when_apply_delete_selection_through_engine_state_history(
    world: &mut CommandWorld,
) -> TestSupportResult<()> {
    let command = prepare_command(Action::DeleteSelection, &world.state)
        .map_err(|e| TestSupportError::expectation(format!("prepare failed: {e}")))?;
    world
        .state
        .apply_document_command(command)
        .map_err(|e| TestSupportError::expectation(format!("apply through engine failed: {e}")))?;
    Ok(())
}

#[when("I undo through EngineState history")]
fn when_undo_through_engine_state_history(world: &mut CommandWorld) -> TestSupportResult<()> {
    world
        .state
        .undo_document()
        .map_err(|e| TestSupportError::expectation(format!("engine undo failed: {e}")))?;
    Ok(())
}

#[when("I redo through EngineState history")]
fn when_redo_through_engine_state_history(world: &mut CommandWorld) -> TestSupportResult<()> {
    world
        .state
        .redo_document()
        .map_err(|e| TestSupportError::expectation(format!("engine redo failed: {e}")))?;
    Ok(())
}

#[when("I end an EngineState history group without begin")]
fn when_end_engine_state_history_group_without_begin(world: &mut CommandWorld) {
    world.history_error = world.state.end_document_history_group().err();
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

#[then("the command should be Reorder")]
fn then_command_is_reorder(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = get_command(world)?;
    match cmd {
        Command::Reorder { .. } => Ok(()),
        _ => Err(TestSupportError::expectation("expected Reorder")),
    }
}

#[then("the command should include one reorder operation")]
fn then_command_has_one_reorder_op(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = get_command(world)?;
    match cmd {
        Command::Reorder { operations } if operations.len() == 1 => Ok(()),
        Command::Reorder { operations } => Err(TestSupportError::expectation(format!(
            "expected 1 operation, got {}",
            operations.len()
        ))),
        _ => Err(TestSupportError::expectation("expected Reorder")),
    }
}

/// Helper: assert document has expected number of shapes.
fn assert_doc_shape_count(world: &CommandWorld, expected: usize) -> TestSupportResult<()> {
    if world.state.document.len() == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected {} shape{}, got {}",
            expected,
            if expected == 1 { "" } else { "s" },
            world.state.document.len()
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
        Some(Ok(_)) => Err(TestSupportError::expectation("expected error, got success")),
        None => Err(TestSupportError::missing("command", "check")),
    }
}

#[then("ending group should fail with NoActiveGroup")]
fn then_ending_group_fails_with_no_active_group(world: &CommandWorld) -> TestSupportResult<()> {
    match world.history_error {
        Some(HistoryError::NoActiveGroup) => Ok(()),
        Some(ref error) => Err(TestSupportError::expectation(format!(
            "expected NoActiveGroup, got {error}"
        ))),
        None => Err(TestSupportError::expectation(
            "expected group boundary operation to fail",
        )),
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

#[then(r#"the inverse name should be "Delete""#)]
fn then_inverse_name_is_delete(world: &CommandWorld) -> TestSupportResult<()> {
    let inverse = world
        .inverse
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("inverse", "check name"))?;

    if inverse.name() == "Delete" {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            r#"expected "Delete", got "{}""#,
            inverse.name()
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

#[scenario(
    path = "tests/features/command.feature",
    name = "Raise selection produces reorder command"
)]
fn raise_selection_produces_reorder_command(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Inverse command has matching name"
)]
fn inverse_command_has_matching_name(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Empty history undo is safe"
)]
fn empty_history_undo_is_safe(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "EngineState history delete-selection round trip"
)]
fn engine_state_history_delete_selection_round_trip(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "EngineState history group boundary reports no active group"
)]
fn engine_state_history_group_boundary_reports_no_active_group(world: CommandWorld) {
    let _ = world;
}
