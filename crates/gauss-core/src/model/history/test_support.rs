//! Test-only snapshots for complete document-history assertions.
//!
//! The public projection keeps `HistoryEntry` private while preserving the
//! ordered `undo_2` representation and its realised cursor position.

use super::DocumentUndoHistory;

/// Test-only snapshot of document-history entries and cursor state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentHistoryState {
    /// Complete ordered command and undo-marker representation.
    pub entries: Vec<String>,
    /// Number of realised entries at the current undo/redo cursor.
    pub cursor: usize,
    /// Number of realised entries currently available to undo.
    pub entry_count: usize,
    /// Whether the current history position can be undone.
    pub can_undo: bool,
    /// Whether the current history position can be redone.
    pub can_redo: bool,
}

impl DocumentUndoHistory {
    /// Return a complete history snapshot for headless test assertions.
    ///
    /// The snapshot records every ordered command and undo marker, together
    /// with the currently realised cursor position.
    #[must_use]
    pub fn state_for_tests(&self) -> DocumentHistoryState {
        let entry_count = self.len();
        DocumentHistoryState {
            entries: self
                .commands
                .iter()
                .map(|entry| format!("{entry:?}"))
                .collect(),
            cursor: entry_count,
            entry_count,
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
        }
    }
}
