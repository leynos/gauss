//! Selection-based edits for Phase 0.
//!
//! These helpers keep selection state in sync with command-driven edits.

use super::Phase0Shell;
use crate::model::{Action, SelectToolState, Selection, prepare_command};

impl Phase0Shell {
    /// Delete the selected shapes through the command pipeline.
    ///
    /// Clears the selection and resets the select tool on success; returns
    /// `false` when the command cannot be prepared or applied.
    pub(super) fn delete_selected_shapes(&mut self) -> bool {
        let command = match prepare_command(Action::DeleteSelection, &self.state) {
            Ok(command) => command,
            Err(error) => {
                log::error!("prepare delete selection command failed: {error}");
                return false;
            }
        };

        if let Err(error) = self.apply_command(command) {
            log::error!("apply delete selection command failed: {error}");
            return false;
        }

        let previous_selection = self.state.selection.clone();
        let new_selection = Selection::empty();
        self.record_selection_change(previous_selection, new_selection.clone());
        self.state.selection = new_selection;
        self.select_tool_state = SelectToolState::Idle;
        true
    }
}
