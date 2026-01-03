//! Z-order reordering for Phase 0.
//!
//! Raise and lower are routed through Commands so undo/redo is consistent.

use crate::model::{Action, prepare_command};

use super::{Phase0Shell, draw::ToolMode};

impl Phase0Shell {
    pub(super) fn raise_selected_shapes(&mut self) -> bool {
        self.apply_reorder_action(Action::RaiseSelection)
    }

    pub(super) fn lower_selected_shapes(&mut self) -> bool {
        self.apply_reorder_action(Action::LowerSelection)
    }

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
