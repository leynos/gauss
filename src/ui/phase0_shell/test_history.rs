//! Test-only document-history inspection for the Phase 0 shell.
//!
//! Headless GPUI scenarios use this focused extension to distinguish history
//! stacks with the same entry count but different undo or redo positions.

use super::Phase0Shell;

impl Phase0Shell {
    /// Return the observable document-history entries and cursor state.
    ///
    /// Headless GPUI tests use this to distinguish stacks with the same entry
    /// count but different undo or redo availability.
    #[must_use]
    pub fn document_history_state_for_tests(&self) -> (usize, bool, bool) {
        (
            self.state.document_history_len(),
            self.state.can_undo_document(),
            self.state.can_redo_document(),
        )
    }
}
