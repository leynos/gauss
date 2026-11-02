//! Selection history support for the Phase 0 shell.
//!
//! Phase 0 keeps selection undo/redo separate from document edits. The intent
//! is to validate the dual-history approach early, before adding more complex
//! selection graphs.

use gpui_component::history::HistoryItem;

use super::Phase0Shell;
use crate::model::{SelectToolState, Selection};

/// A single selection transition recorded for undo and redo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectionChange {
    /// Selection before the change.
    pub(super) from: Selection,
    /// Selection after the change.
    pub(super) to: Selection,
}

/// History entry wrapping a selection change with its group version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectionHistoryItem {
    /// Version assigned by the history container; zero until grouped.
    version: usize,
    /// Selection transition this entry replays.
    pub(super) change: SelectionChange,
}

impl HistoryItem for SelectionHistoryItem {
    fn version(&self) -> usize { self.version }

    fn set_version(&mut self, version: usize) { self.version = version; }
}

impl SelectionHistoryItem {
    /// Create an unversioned history item for `change`.
    pub(super) const fn new(change: SelectionChange) -> Self { Self { version: 0, change } }
}

impl Phase0Shell {
    /// Record a selection transition, ignoring no-op changes.
    pub(super) fn record_selection_change(&mut self, from: Selection, to: Selection) {
        if from == to {
            return;
        }

        self.selection_history
            .push(SelectionHistoryItem::new(SelectionChange { from, to }));
    }

    /// Restore the selection preceding the last recorded change.
    ///
    /// Resets the select tool to idle; does nothing when history is empty.
    pub(super) fn undo_selection(&mut self) {
        let Some(group) = self.selection_history.undo() else {
            return;
        };

        if let Some(last) = group.last() {
            self.state.selection = last.change.from.clone();
        }

        self.select_tool_state = SelectToolState::Idle;
    }

    /// Reapply the selection following the last undone change.
    ///
    /// Resets the select tool to idle; does nothing when nothing is undone.
    pub(super) fn redo_selection(&mut self) {
        let Some(group) = self.selection_history.redo() else {
            return;
        };

        if let Some(last) = group.last() {
            self.state.selection = last.change.to.clone();
        }

        self.select_tool_state = SelectToolState::Idle;
    }
}
