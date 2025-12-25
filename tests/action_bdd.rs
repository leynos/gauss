//! Behaviour tests for Action categorisation.
//!
//! These tests use `rstest-bdd` to validate that actions are correctly
//! categorised for dispatch routing.

use gauss::model::{Action, ActionKind};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then};
use test_support::{TestSupportError, TestSupportResult};

/// World state for action BDD tests.
#[derive(Default)]
struct ActionWorld {
    action: Option<Action>,
}

#[fixture]
fn world() -> ActionWorld {
    ActionWorld::default()
}

// === Given steps ===

#[given("the action DeleteSelection")]
fn given_delete_selection(world: &mut ActionWorld) {
    world.action = Some(Action::DeleteSelection);
}

#[given("the action SelectAll")]
fn given_select_all(world: &mut ActionWorld) {
    world.action = Some(Action::SelectAll);
}

#[given("the action ActivatePenTool")]
fn given_activate_pen_tool(world: &mut ActionWorld) {
    world.action = Some(Action::ActivatePenTool);
}

#[given("the action Undo")]
fn given_undo(world: &mut ActionWorld) {
    world.action = Some(Action::Undo);
}

// === Then steps ===

#[then("its kind should be Document")]
fn then_kind_is_document(world: &ActionWorld) -> TestSupportResult<()> {
    let action = world
        .action
        .ok_or_else(|| TestSupportError::missing("action", "kind check"))?;
    if action.kind() != ActionKind::Document {
        return Err(TestSupportError::expectation(format!(
            "Expected {:?} to have Document kind, got {:?}",
            action,
            action.kind()
        )));
    }
    Ok(())
}

#[then("its kind should be Editor")]
fn then_kind_is_editor(world: &ActionWorld) -> TestSupportResult<()> {
    let action = world
        .action
        .ok_or_else(|| TestSupportError::missing("action", "kind check"))?;
    if action.kind() != ActionKind::Editor {
        return Err(TestSupportError::expectation(format!(
            "Expected {:?} to have Editor kind, got {:?}",
            action,
            action.kind()
        )));
    }
    Ok(())
}

#[then("it should require a selection")]
fn then_requires_selection(world: &ActionWorld) -> TestSupportResult<()> {
    let action = world
        .action
        .ok_or_else(|| TestSupportError::missing("action", "selection check"))?;
    if !action.requires_selection() {
        return Err(TestSupportError::expectation(format!(
            "Expected {action:?} to require a selection"
        )));
    }
    Ok(())
}

#[then("it should not require a selection")]
fn then_does_not_require_selection(world: &ActionWorld) -> TestSupportResult<()> {
    let action = world
        .action
        .ok_or_else(|| TestSupportError::missing("action", "selection check"))?;
    if action.requires_selection() {
        return Err(TestSupportError::expectation(format!(
            "Expected {action:?} to not require a selection"
        )));
    }
    Ok(())
}

// === Scenario bindings ===

#[scenario(path = "tests/features/action.feature", name = "Document action kind")]
fn document_action_kind(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Editor action kind for selection"
)]
fn editor_action_kind_for_selection(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Editor action kind for tools"
)]
fn editor_action_kind_for_tools(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Editor action kind for history"
)]
fn editor_action_kind_for_history(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Action requires selection"
)]
fn action_requires_selection(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Action does not require selection"
)]
fn action_does_not_require_selection(world: ActionWorld) {
    let _ = world;
}
