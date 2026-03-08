//! Tool-command application helpers for `Phase0Shell`.
//!
//! This module keeps tool state transitions and command application out of the
//! chrome layout module so each module remains small and focused.

use crate::model::{
    Command, SelectPointerHit, SelectToolState, Tool, ToolCommand, ToolInputEvent, ToolMode,
    ToolModeFsm, UserError, apply_select_drag_preview, restore_select_drag_preview,
};

use super::{Phase0Shell, draw::DrawEdgeMode};

impl Phase0Shell {
    /// Activate draw mode with an optional edge mode override.
    ///
    /// `edge_mode` is forwarded to the draw activation event; `Some(mode)`
    /// requests that mode, while `None` keeps draw mode activation without an
    /// explicit override. Returns `true` when applying emitted tool commands
    /// changes shell state or document state.
    pub(super) fn activate_draw_tool(&mut self, edge_mode: Option<DrawEdgeMode>) -> bool {
        self.handle_tool_input_event(ToolInputEvent::ActivateDraw { edge_mode })
    }

    /// Activate manipulate/select mode through the tool FSM.
    ///
    /// Returns `true` when emitted commands mutate state (for example tool
    /// mode, selection, or select-tool runtime state).
    pub(super) fn activate_select_tool(&mut self) -> bool {
        self.handle_tool_input_event(ToolInputEvent::ActivateManipulate)
    }

    /// Evaluate one tool input event and apply all resulting commands.
    ///
    /// The transition is computed via `ToolModeFsm.transition(...)`, and
    /// command application may mutate document/editor state. Returns `true`
    /// when at least one emitted command changes state.
    pub(super) fn handle_tool_input_event(&mut self, event: ToolInputEvent) -> bool {
        let transition = ToolModeFsm.transition(self.state.tool_mode, self.state.edge_mode, event);
        self.apply_tool_commands(transition.commands)
    }

    /// Apply a sequence of tool commands in order.
    ///
    /// Accepts any `IntoIterator<Item = ToolCommand>`, mutates shell/document
    /// state per command, and returns whether any command changed state. On
    /// command errors this logs the error, records `last_history_error`, and
    /// returns the accumulated change state for commands applied before failure.
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
                    // Preserve "state changed" semantics so callers still
                    // trigger redraw when earlier commands mutated state.
                    return did_change;
                }
            }
        }

        did_change
    }

    fn apply_tool_command(&mut self, command: ToolCommand) -> Result<bool, UserError> {
        Ok(match command {
            ToolCommand::ApplyDocumentCommand(document_command) => {
                self.apply_document_tool_command(*document_command)?
            }
            ToolCommand::SetToolMode(mode) => self.set_tool_mode_if_changed(mode),
            ToolCommand::SetEdgeMode(mode) => {
                if self.state.edge_mode == mode {
                    false
                } else {
                    self.state.edge_mode = mode;
                    true
                }
            }
            ToolCommand::SetActivePath(path) => {
                if self.state.active_path == path {
                    false
                } else {
                    self.state.active_path = path;
                    true
                }
            }
            ToolCommand::SetSelection(selection) => {
                if self.state.selection == selection {
                    false
                } else {
                    self.state.selection = selection;
                    true
                }
            }
            ToolCommand::RecordSelectionChange { from, to } => {
                if from == to {
                    false
                } else {
                    self.record_selection_change(from, to);
                    true
                }
            }
            ToolCommand::SetSelectToolState(state) => self.set_select_tool_state_if_changed(state),
            ToolCommand::PreviewSelectDrag { cursor_world } => {
                let did_change = apply_select_drag_preview(
                    &mut self.state.document,
                    &self.select_tool_state,
                    cursor_world,
                );
                if did_change {
                    self.invalidate_hover_cache();
                }
                did_change
            }
            ToolCommand::RestoreSelectDragPreview => {
                let did_change =
                    restore_select_drag_preview(&mut self.state.document, &self.select_tool_state);
                if did_change {
                    self.invalidate_hover_cache();
                }
                did_change
            }
        })
    }

    fn apply_document_tool_command(&mut self, command: Command) -> Result<bool, UserError> {
        self.apply_command(command)?;
        self.invalidate_hover_cache();
        Ok(true)
    }

    const fn invalidate_hover_cache(&mut self) {
        self.document_generation = self.document_generation.wrapping_add(1);
        self.hover_cache = None;
    }

    fn set_tool_mode_if_changed(&mut self, mode: ToolMode) -> bool {
        let mut did_change = false;

        if self.state.tool_mode != mode {
            self.state.tool_mode = mode;
            did_change = true;
        }

        if mode != ToolMode::Manipulate && self.select_tool_state != SelectToolState::Idle {
            restore_select_drag_preview(&mut self.state.document, &self.select_tool_state);
            self.select_tool_state = SelectToolState::Idle;
            did_change = true;
        }

        if mode != ToolMode::Manipulate && self.hover_hit != SelectPointerHit::None {
            self.hover_hit = SelectPointerHit::None;
            self.hover_cache = None;
            did_change = true;
        }

        did_change
    }

    fn set_select_tool_state_if_changed(&mut self, state: SelectToolState) -> bool {
        if self.select_tool_state == state {
            return false;
        }

        self.select_tool_state = state;
        true
    }
}
