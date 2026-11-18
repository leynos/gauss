//! Replay state for one undo or redo group.
//!
//! A group replays entry by entry: the first failure decides the group's
//! result, while the remaining entries still run so each one reports its own
//! error. Keeping the document borrow, the replayed action and the first
//! failure together lets the loop take only the entry iterator.

use undo_2::Action;

use super::{DocumentUndoHistory, HistoryEntry, HistoryError, ReplayDirection};
use crate::model::Document;

/// State carried while one undo or redo group replays entry by entry.
pub(super) struct Replay<'a> {
    /// The document each entry is replayed against.
    doc: &'a mut Document,
    /// Whether the group is undoing or redoing.
    action: Action,
    /// The direction the history index is moving.
    direction: ReplayDirection,
    /// The first failure seen, which decides the group's result.
    first_error: Option<HistoryError>,
}

impl<'a> Replay<'a> {
    /// Prepares a replay of one group against `doc`.
    pub(super) const fn new(
        doc: &'a mut Document,
        action: Action,
        direction: ReplayDirection,
    ) -> Self {
        Self {
            doc,
            action,
            direction,
            first_error: None,
        }
    }

    /// Returns the first failure seen, if any.
    pub(super) fn first_error(self) -> Option<HistoryError> { self.first_error }

    /// Replays entries in the supplied order, keeping the first error.
    pub(super) fn run<'entry>(&mut self, entries: impl Iterator<Item = &'entry HistoryEntry>) {
        for entry in entries {
            let result =
                DocumentUndoHistory::apply_entry(self.doc, self.action, entry, self.direction);
            DocumentUndoHistory::capture_first_error(&mut self.first_error, result);
        }
    }
}
