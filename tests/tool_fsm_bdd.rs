//! Behaviour tests for Tool mode FSM command emission.
//!
//! These tests validate command-based tool transitions for happy and unhappy
//! paths using `rstest-bdd` scenarios.

use gauss::model::{
    EdgeMode, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolModeFsm, ToolTransition,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

/// World state for tool FSM BDD tests.
#[derive(Default)]
struct ToolFsmWorld {
    mode: ToolMode,
    edge_mode: EdgeMode,
    input_event: Option<ToolInputEvent>,
    transition: Option<ToolTransition>,
}

#[fixture]
fn world() -> ToolFsmWorld {
    ToolFsmWorld {
        mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        input_event: None,
        transition: None,
    }
}

// === Given steps ===

#[given("the current tool mode is Draw")]
fn given_tool_mode_draw(world: &mut ToolFsmWorld) {
    world.mode = ToolMode::Draw;
}

#[given("the current tool mode is Manipulate")]
fn given_tool_mode_manipulate(world: &mut ToolFsmWorld) {
    world.mode = ToolMode::Manipulate;
}

#[given("the current edge mode is Line")]
fn given_edge_mode_line(world: &mut ToolFsmWorld) {
    world.edge_mode = EdgeMode::Line;
}

#[given("the current edge mode is BezierAuto")]
fn given_edge_mode_bezier(world: &mut ToolFsmWorld) {
    world.edge_mode = EdgeMode::BezierAuto;
}

#[given("the input event is EscapePressed")]
fn given_input_escape(world: &mut ToolFsmWorld) {
    world.input_event = Some(ToolInputEvent::EscapePressed);
}

#[given("the input event is ToggleEdgeMode")]
fn given_input_toggle_edge(world: &mut ToolFsmWorld) {
    world.input_event = Some(ToolInputEvent::ToggleEdgeMode);
}

#[given("the input event is ActivateDrawBezier")]
fn given_input_activate_draw_bezier(world: &mut ToolFsmWorld) {
    world.input_event = Some(ToolInputEvent::ActivateDraw {
        edge_mode: Some(EdgeMode::BezierAuto),
    });
}

#[given("the input event is ActivateDraw")]
fn given_input_activate_draw(world: &mut ToolFsmWorld) {
    world.input_event = Some(ToolInputEvent::ActivateDraw { edge_mode: None });
}

#[given("the input event is ClosePathCommitted")]
fn given_input_close_path_committed(world: &mut ToolFsmWorld) {
    world.input_event = Some(ToolInputEvent::ClosePathCommitted);
}

// === When steps ===

#[when("the tool transition is evaluated")]
fn when_transition_is_evaluated(world: &mut ToolFsmWorld) -> TestSupportResult<()> {
    let input_event = world
        .input_event
        .ok_or_else(|| TestSupportError::missing("input_event", "transition evaluation"))?;

    let fsm = ToolModeFsm;
    world.transition = Some(fsm.transition(world.mode, world.edge_mode, input_event));
    Ok(())
}

// === Helper functions ===

fn transition(world: &ToolFsmWorld) -> TestSupportResult<&ToolTransition> {
    world
        .transition
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("transition", "assertion"))
}

fn assert_contains_command(world: &ToolFsmWorld, expected: &ToolCommand) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.iter().any(|actual| actual == expected) {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "Expected command {expected:?}; actual commands: {commands:?}"
    )))
}

fn assert_command_count(world: &ToolFsmWorld, expected_count: usize) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.len() == expected_count {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "Expected {expected_count} commands; got {} ({commands:?})",
        commands.len()
    )))
}

// === Then steps ===

#[then("it should emit SetToolMode Manipulate")]
fn then_emit_set_tool_mode_manipulate(world: &ToolFsmWorld) -> TestSupportResult<()> {
    assert_contains_command(world, &ToolCommand::SetToolMode(ToolMode::Manipulate))
}

#[then("it should emit SetToolMode Draw")]
fn then_emit_set_tool_mode_draw(world: &ToolFsmWorld) -> TestSupportResult<()> {
    assert_contains_command(world, &ToolCommand::SetToolMode(ToolMode::Draw))
}

#[then("it should emit SetActivePath None")]
fn then_emit_set_active_path_none(world: &ToolFsmWorld) -> TestSupportResult<()> {
    assert_contains_command(world, &ToolCommand::SetActivePath(None))
}

#[then("it should emit SetEdgeMode BezierAuto")]
fn then_emit_set_edge_mode_bezier(world: &ToolFsmWorld) -> TestSupportResult<()> {
    assert_contains_command(world, &ToolCommand::SetEdgeMode(EdgeMode::BezierAuto))
}

#[then("it should emit no commands")]
fn then_emit_no_commands(world: &ToolFsmWorld) -> TestSupportResult<()> {
    assert_command_count(world, 0)
}

#[then("it should emit exactly {count:usize} commands")]
fn then_emit_exactly_n_commands(world: &ToolFsmWorld, count: usize) -> TestSupportResult<()> {
    assert_command_count(world, count)
}

#[then("it should emit exactly {count:usize} command")]
fn then_emit_exactly_one_command_alias(
    world: &ToolFsmWorld,
    count: usize,
) -> TestSupportResult<()> {
    assert_command_count(world, count)
}

// === Scenario bindings ===

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Escape from draw enters manipulate and clears active path"
)]
fn escape_from_draw_transitions_to_manipulate(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Escape from manipulate enters draw"
)]
fn escape_from_manipulate_transitions_to_draw(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Toggle edge mode in draw emits edge command"
)]
fn toggle_edge_in_draw_emits_command(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Toggle edge mode in manipulate emits nothing"
)]
fn toggle_edge_in_manipulate_emits_no_commands(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Activate draw from manipulate with explicit edge mode"
)]
fn activate_draw_from_manipulate_emits_mode_and_edge(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Activate draw from manipulate without explicit edge mode"
)]
fn activate_draw_without_edge_from_manipulate_emits_mode(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Activate draw in draw with explicit edge mode only changes edge"
)]
fn activate_draw_in_draw_with_edge_only_emits_edge(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Close path committed from draw enters manipulate and clears active path"
)]
fn close_path_committed_from_draw_emits_mode_and_path_commands(world: ToolFsmWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/tool_fsm.feature",
    name = "Close path committed in manipulate emits nothing"
)]
fn close_path_committed_outside_draw_emits_no_commands(world: ToolFsmWorld) {
    let _ = world;
}
