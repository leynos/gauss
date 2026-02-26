//! Unit tests for the tool FSM model surface.

use super::{Command, EdgeMode, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolModeFsm};
use rstest::rstest;

#[rstest]
fn tool_mode_has_label() {
    assert_eq!(ToolMode::Draw.label(), "Draw");
    assert_eq!(ToolMode::Manipulate.label(), "Manipulate");
}

#[rstest]
fn tool_mode_default_is_draw() {
    assert_eq!(ToolMode::default(), ToolMode::Draw);
}

#[rstest]
fn edge_mode_has_label() {
    assert_eq!(EdgeMode::Line.label(), "Line");
    assert_eq!(EdgeMode::BezierAuto.label(), "Bezier (auto)");
}

#[rstest]
fn edge_mode_default_is_line() {
    assert_eq!(EdgeMode::default(), EdgeMode::Line);
}

#[rstest]
fn edge_mode_toggle_switches() {
    assert_eq!(EdgeMode::Line.toggle(), EdgeMode::BezierAuto);
    assert_eq!(EdgeMode::BezierAuto.toggle(), EdgeMode::Line);
}

#[rstest]
fn tool_mode_is_copy() {
    let mode = ToolMode::Draw;
    let copied = mode;
    assert_eq!(mode, copied);
}

#[rstest]
fn edge_mode_is_copy() {
    let mode = EdgeMode::Line;
    let copied = mode;
    assert_eq!(mode, copied);
}

#[rstest]
fn activate_draw_from_manipulate_sets_draw_and_requested_edge_mode() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::ActivateDraw {
            edge_mode: Some(EdgeMode::BezierAuto),
        },
    );

    assert_eq!(
        transition.commands,
        vec![
            ToolCommand::SetToolMode(ToolMode::Draw),
            ToolCommand::SetEdgeMode(EdgeMode::BezierAuto),
        ]
    );
}

#[rstest]
fn activate_draw_from_manipulate_without_edge_request_only_sets_draw_mode() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::ActivateDraw { edge_mode: None },
    );

    assert_eq!(
        transition.commands,
        vec![ToolCommand::SetToolMode(ToolMode::Draw)]
    );
}

#[rstest]
fn activate_draw_from_manipulate_with_same_edge_request_only_sets_draw_mode() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::ActivateDraw {
            edge_mode: Some(EdgeMode::Line),
        },
    );

    assert_eq!(
        transition.commands,
        vec![ToolCommand::SetToolMode(ToolMode::Draw)]
    );
}

#[rstest]
fn activate_draw_in_draw_without_edge_request_is_noop() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::ActivateDraw { edge_mode: None },
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn activate_draw_in_draw_with_same_edge_request_is_noop() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::ActivateDraw {
            edge_mode: Some(EdgeMode::Line),
        },
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn activate_draw_in_draw_with_edge_request_only_sets_edge_mode() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::ActivateDraw {
            edge_mode: Some(EdgeMode::BezierAuto),
        },
    );

    assert_eq!(
        transition.commands,
        vec![ToolCommand::SetEdgeMode(EdgeMode::BezierAuto)]
    );
}

#[rstest]
fn activate_manipulate_emits_mode_and_active_path_commands() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::ActivateManipulate,
    );

    assert_eq!(
        transition.commands,
        vec![
            ToolCommand::SetToolMode(ToolMode::Manipulate),
            ToolCommand::SetActivePath(None),
        ]
    );
}

#[rstest]
fn escape_from_draw_switches_to_manipulate_and_clears_active_path() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::EscapePressed,
    );

    assert_eq!(
        transition.commands,
        vec![
            ToolCommand::SetToolMode(ToolMode::Manipulate),
            ToolCommand::SetActivePath(None),
        ]
    );
}

#[rstest]
fn escape_from_manipulate_switches_to_draw() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::EscapePressed,
    );

    assert_eq!(
        transition.commands,
        vec![ToolCommand::SetToolMode(ToolMode::Draw)]
    );
}

#[rstest]
fn toggle_edge_mode_in_draw_emits_set_edge_mode_command() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::ToggleEdgeMode,
    );

    assert_eq!(
        transition.commands,
        vec![ToolCommand::SetEdgeMode(EdgeMode::BezierAuto)]
    );
}

#[rstest]
fn toggle_edge_mode_outside_draw_is_noop() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::ToggleEdgeMode,
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn close_path_committed_switches_to_manipulate_and_clears_active_path() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Draw,
        EdgeMode::BezierAuto,
        ToolInputEvent::ClosePathCommitted,
    );

    assert_eq!(
        transition.commands,
        vec![
            ToolCommand::SetToolMode(ToolMode::Manipulate),
            ToolCommand::SetActivePath(None),
        ]
    );
}

#[rstest]
fn close_path_committed_outside_draw_is_noop() {
    let fsm = ToolModeFsm;

    let transition = fsm.transition(
        ToolMode::Manipulate,
        EdgeMode::BezierAuto,
        ToolInputEvent::ClosePathCommitted,
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn tool_command_can_wrap_document_command() {
    let command = Command::DeleteShapes { targets: vec![] };

    let wrapped = ToolCommand::ApplyDocumentCommand(Box::new(command.clone()));

    assert_eq!(
        wrapped,
        ToolCommand::ApplyDocumentCommand(Box::new(command))
    );
}
