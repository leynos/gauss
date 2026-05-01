//! Mode-switching FSM for draw and manipulate tools.

use super::{EdgeMode, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolTransition};

/// FSM for draw/manipulate mode transitions.
///
/// Maps high-level tool input events to [`ToolTransition`] values. Pointer and
/// selection events are handled by the concrete tool FSMs, so this mode FSM
/// treats them as no-ops.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToolModeFsm;

fn activate_draw_transition(
    current_mode: ToolMode,
    current_edge_mode: EdgeMode,
    edge_mode: Option<EdgeMode>,
) -> ToolTransition {
    let mut commands = Vec::new();
    if current_mode != ToolMode::Draw {
        commands.push(ToolCommand::SetToolMode(ToolMode::Draw));
    }
    if let Some(next_edge_mode) = edge_mode
        && next_edge_mode != current_edge_mode
    {
        commands.push(ToolCommand::SetEdgeMode(next_edge_mode));
    }
    ToolTransition::with_commands(commands)
}

fn escape_transition(current_mode: ToolMode) -> ToolTransition {
    match current_mode {
        ToolMode::Draw => ToolTransition::with_commands([
            ToolCommand::SetToolMode(ToolMode::Manipulate),
            ToolCommand::SetActivePath(None),
        ]),
        ToolMode::Manipulate => {
            ToolTransition::with_commands([ToolCommand::SetToolMode(ToolMode::Draw)])
        }
    }
}

impl Tool for ToolModeFsm {
    /// Map one [`ToolInputEvent`] to commands for the draw/manipulate FSM.
    ///
    /// `current_mode` and `current_edge_mode` describe the shell state before
    /// the event. Pointer and selection events return an empty transition
    /// because they are handled by pen/select tool logic.
    fn transition(
        &self,
        current_mode: ToolMode,
        current_edge_mode: EdgeMode,
        event: ToolInputEvent,
    ) -> ToolTransition {
        match event {
            ToolInputEvent::ActivateDraw { edge_mode } => {
                activate_draw_transition(current_mode, current_edge_mode, edge_mode)
            }
            ToolInputEvent::ActivateManipulate => ToolTransition::with_commands([
                ToolCommand::SetToolMode(ToolMode::Manipulate),
                ToolCommand::SetActivePath(None),
            ]),
            ToolInputEvent::EscapePressed => escape_transition(current_mode),
            ToolInputEvent::ToggleEdgeMode => {
                if current_mode != ToolMode::Draw {
                    return ToolTransition::default();
                }
                ToolTransition::with_commands([ToolCommand::SetEdgeMode(
                    current_edge_mode.toggle(),
                )])
            }
            ToolInputEvent::ClosePathCommitted => {
                if current_mode != ToolMode::Draw {
                    return ToolTransition::default();
                }
                ToolTransition::with_commands([
                    ToolCommand::SetToolMode(ToolMode::Manipulate),
                    ToolCommand::SetActivePath(None),
                ])
            }
            ToolInputEvent::PenCanvasClicked { .. }
            | ToolInputEvent::SelectPointerDown { .. }
            | ToolInputEvent::SelectPointerMove { .. }
            | ToolInputEvent::SelectPointerUp { .. } => ToolTransition::default(),
        }
    }
}
