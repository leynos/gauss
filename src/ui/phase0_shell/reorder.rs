//! Z-order reordering for Phase 0.
//!
//! Raise and lower are routed through Commands so undo/redo is consistent.

use super::{Phase0Shell, draw::ToolMode};
use crate::model::{Action, prepare_command};

impl Phase0Shell {
    /// Move the selected shapes one step up in z-order.
    pub(super) fn raise_selected_shapes(&mut self) -> bool {
        self.apply_reorder_action(Action::RaiseSelection)
    }

    /// Move the selected shapes one step down in z-order.
    pub(super) fn lower_selected_shapes(&mut self) -> bool {
        self.apply_reorder_action(Action::LowerSelection)
    }

    /// Prepare and apply a reorder action in manipulate mode.
    ///
    /// Returns `false` outside manipulate mode, when the command cannot be
    /// prepared, or when applying it fails.
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
