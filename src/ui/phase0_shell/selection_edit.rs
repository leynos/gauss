//! Selection-based edits for Phase 0.
//!
//! These helpers keep selection state in sync with command-driven edits.

use crate::model::{Action, Selection, prepare_command};

use super::Phase0Shell;

impl Phase0Shell {
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
        self.drag_state = None;
        true
    }
}
