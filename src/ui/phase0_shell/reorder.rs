//! Z-order reordering for Phase 0.
//!
//! Raise and lower are routed through Commands so undo/redo is consistent.

use crate::model::{Action, prepare_command};

use super::{Phase0Shell, draw::ToolMode};

impl Phase0Shell {
    /// Raise the selected shapes through the command history.
    ///
    /// Returns `false` when the shell is outside Manipulate mode or the command
    /// cannot be prepared or applied.
    ///
    /// For example, with a valid selection in Manipulate mode, a successful raise
    /// applies the command and returns `true`.
    pub(super) fn raise_selected_shapes(&mut self) -> bool {
        self.apply_reorder_action(Action::RaiseSelection)
    }

    /// Lower the selected shapes through the command history.
    ///
    /// Returns `false` when the shell is outside Manipulate mode or the command
    /// cannot be prepared or applied.
    ///
    /// For example, with a valid selection in Manipulate mode, a successful lower
    /// applies the command and returns `true`.
    pub(super) fn lower_selected_shapes(&mut self) -> bool {
        self.apply_reorder_action(Action::LowerSelection)
    }

    /// Prepare and apply a reorder action when the shell is in Manipulate mode.
    ///
    /// For example, a reorder request in Draw mode leaves the document unchanged
    /// and returns `false`.
    fn apply_reorder_action(&mut self, action: Action) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let Ok(command) = prepare_command(action, &self.state) else {
            return false;
        };

        self.apply_command(command).is_ok()
    }
}
