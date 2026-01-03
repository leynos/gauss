//! Segment kind toggling for Phase 0.
//!
//! Segment toggles are routed through Commands so undo/redo remains consistent.

use crate::model::{Action, prepare_command};

use super::Phase0Shell;

impl Phase0Shell {
    pub(super) fn toggle_selected_segments_kind(&mut self) -> bool {
        let Ok(command) = prepare_command(Action::ToggleSegmentKind, &self.state) else {
            return false;
        };

        self.apply_command(command).is_ok()
    }
}
