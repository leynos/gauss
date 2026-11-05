//! Segment kind toggling for Phase 0.
//!
//! Segment toggles are routed through Commands so undo/redo remains consistent.

use super::Phase0Shell;
use crate::model::{Action, prepare_command};

impl Phase0Shell {
    /// Toggle the line kind (straight or curved) of the selected segments.
    ///
    /// Returns `false` when the command cannot be prepared or applied.
    pub(super) fn toggle_selected_segments_kind(&mut self) -> bool {
        let Ok(command) = prepare_command(Action::ToggleSegmentKind, &self.state) else {
            return false;
        };

        self.apply_command(command).is_ok()
    }
}
