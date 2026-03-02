//! Tool-command application helpers for `Phase0Shell`.
//!
//! This module keeps tool state transitions and command application out of the
//! chrome layout module so each module remains small and focused.

use crate::model::{
    Command, SelectToolState, Selection, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode,
    ToolModeFsm, UserError, Vec2, apply_select_drag_preview, restore_select_drag_preview,
};

use super::{Phase0Shell, draw::DrawEdgeMode};

impl Phase0Shell {
    pub(super) fn activate_draw_tool(&mut self, edge_mode: Option<DrawEdgeMode>) -> bool {
        self.handle_tool_input_event(ToolInputEvent::ActivateDraw { edge_mode })
    }

    pub(super) fn activate_select_tool(&mut self) -> bool {
        self.handle_tool_input_event(ToolInputEvent::ActivateManipulate)
    }

    pub(super) fn handle_tool_input_event(&mut self, event: ToolInputEvent) -> bool {
        let transition = ToolModeFsm.transition(self.state.tool_mode, self.state.edge_mode, event);
        self.apply_tool_commands(transition.commands)
    }

    pub(super) fn apply_tool_commands(
        &mut self,
        commands: impl IntoIterator<Item = ToolCommand>,
    ) -> bool {
        let mut did_change = false;

        for command in commands {
            match self.apply_tool_command(command) {
                Ok(command_changed) => {
                    did_change |= command_changed;
                }
                Err(error) => {
                    log::error!("{error}");
                    self.last_history_error = Some(error.to_string());
                    return did_change;
                }
            }
        }

        did_change
    }

    fn apply_tool_command(&mut self, command: ToolCommand) -> Result<bool, UserError> {
        match command {
            ToolCommand::ApplyDocumentCommand(document_command) => {
                self.apply_document_tool_command(*document_command)
            }
            ToolCommand::SetToolMode(mode) => Ok(self.set_tool_mode_if_changed(mode)),
            ToolCommand::SetEdgeMode(mode) => Ok(self.set_edge_mode_if_changed(mode)),
            ToolCommand::SetActivePath(path) => Ok(self.set_active_path_if_changed(path)),
            ToolCommand::SetSelection(selection) => Ok(self.set_selection_if_changed(selection)),
            ToolCommand::RecordSelectionChange { from, to } => {
                Ok(self.record_selection_change_if_changed(from, to))
            }
            ToolCommand::SetSelectToolState(state) => {
                Ok(self.set_select_tool_state_if_changed(state))
            }
            ToolCommand::PreviewSelectDrag { cursor_world } => {
                Ok(self.preview_select_drag_if_possible(cursor_world))
            }
            ToolCommand::RestoreSelectDragPreview => Ok(self.restore_select_drag_if_possible()),
        }
    }

    fn apply_document_tool_command(&mut self, command: Command) -> Result<bool, UserError> {
        self.apply_command(command)?;
        Ok(true)
    }

    fn set_tool_mode_if_changed(&mut self, mode: ToolMode) -> bool {
        let mut did_change = false;

        if self.state.tool_mode != mode {
            self.state.tool_mode = mode;
            did_change = true;
        }

        if mode != ToolMode::Manipulate && self.select_tool_state != SelectToolState::Idle {
            self.select_tool_state = SelectToolState::Idle;
            did_change = true;
        }

        did_change
    }

    fn set_edge_mode_if_changed(&mut self, mode: DrawEdgeMode) -> bool {
        if self.state.edge_mode == mode {
            return false;
        }
        self.state.edge_mode = mode;
        true
    }

    fn set_active_path_if_changed(&mut self, path: Option<ShapeId>) -> bool {
        if self.state.active_path == path {
            return false;
        }
        self.state.active_path = path;
        true
    }

    fn set_selection_if_changed(&mut self, selection: Selection) -> bool {
        if self.state.selection == selection {
            return false;
        }
        self.state.selection = selection;
        true
    }

    fn record_selection_change_if_changed(&mut self, from: Selection, to: Selection) -> bool {
        if from == to {
            return false;
        }
        self.record_selection_change(from, to);
        true
    }

    fn set_select_tool_state_if_changed(&mut self, state: SelectToolState) -> bool {
        if self.select_tool_state == state {
            return false;
        }
        self.select_tool_state = state;
        true
    }

    fn preview_select_drag_if_possible(&mut self, cursor_world: Vec2) -> bool {
        apply_select_drag_preview(
            &mut self.state.document,
            &self.select_tool_state,
            cursor_world,
        )
    }

    fn restore_select_drag_if_possible(&mut self) -> bool {
        restore_select_drag_preview(&mut self.state.document, &self.select_tool_state)
    }
}
