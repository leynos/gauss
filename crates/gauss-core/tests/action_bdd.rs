//! Behaviour tests for Action categorization.
//!
//! These tests use `rstest-bdd` to validate that actions are correctly
//! categorized for dispatch routing.

use gauss_core::model::{Action, ActionKind};
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

#[given("the action Redo")]
fn given_redo(world: &mut ActionWorld) {
    world.action = Some(Action::Redo);
}

#[given("the action DeselectAll")]
fn given_deselect_all(world: &mut ActionWorld) {
    world.action = Some(Action::DeselectAll);
}

#[given("the action ActivateSelectTool")]
fn given_activate_select_tool(world: &mut ActionWorld) {
    world.action = Some(Action::ActivateSelectTool);
}

// === Helper functions ===

fn get_action(world: &ActionWorld, context: &str) -> TestSupportResult<Action> {
    world
        .action
        .ok_or_else(|| TestSupportError::missing("action", context))
}

fn assert_kind(action: Action, expected: ActionKind) -> TestSupportResult<()> {
    if action.kind() != expected {
        return Err(TestSupportError::expectation(format!(
            "Expected {action:?} to have {expected:?} kind, got {:?}",
            action.kind()
        )));
    }
    Ok(())
}

fn assert_requires_selection(action: Action, expected: bool) -> TestSupportResult<()> {
    if action.requires_selection() != expected {
        let msg = if expected {
            format!("Expected {action:?} to require a selection")
        } else {
            format!("Expected {action:?} to not require a selection")
        };
        return Err(TestSupportError::expectation(msg));
    }
    Ok(())
}

// === Then steps ===

#[then("its kind should be Document")]
fn then_kind_is_document(world: &ActionWorld) -> TestSupportResult<()> {
    assert_kind(get_action(world, "kind check")?, ActionKind::Document)
}

#[then("its kind should be Editor")]
fn then_kind_is_editor(world: &ActionWorld) -> TestSupportResult<()> {
    assert_kind(get_action(world, "kind check")?, ActionKind::Editor)
}

#[then("it should require a selection")]
fn then_requires_selection(world: &ActionWorld) -> TestSupportResult<()> {
    assert_requires_selection(get_action(world, "selection check")?, true)
}

#[then("it should not require a selection")]
fn then_does_not_require_selection(world: &ActionWorld) -> TestSupportResult<()> {
    assert_requires_selection(get_action(world, "selection check")?, false)
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

#[scenario(
    path = "tests/features/action.feature",
    name = "Editor action kind for redo"
)]
fn editor_action_kind_for_redo(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Editor action kind for deselection"
)]
fn editor_action_kind_for_deselection(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Editor action kind for select tool"
)]
fn editor_action_kind_for_select_tool(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "DeselectAll does not require selection"
)]
fn deselect_all_does_not_require_selection(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "ActivateSelectTool does not require selection"
)]
fn activate_select_tool_does_not_require_selection(world: ActionWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/action.feature",
    name = "Redo does not require selection"
)]
fn redo_does_not_require_selection(world: ActionWorld) {
    let _ = world;
}
