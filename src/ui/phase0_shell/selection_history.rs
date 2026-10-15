//! Selection history support for the Phase 0 shell.
//!
//! Phase 0 keeps selection undo/redo separate from document edits. The intent
//! is to validate the dual-history approach early, before adding more complex
//! selection graphs.

use gpui_component::history::HistoryItem;

use crate::model::SelectToolState;
use crate::model::Selection;

use super::Phase0Shell;

/// Captures the selection snapshots needed to undo or redo one selection edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectionChange {
    /// Selection before the change.
    pub(super) from: Selection,
    /// Selection after the change.
    pub(super) to: Selection,
}

/// One selection change stored in the shell's separate selection history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectionHistoryItem {
    /// Version assigned by the history stack.
    version: usize,
    /// Selection snapshots restored by undo and redo.
    pub(super) change: SelectionChange,
}

impl HistoryItem for SelectionHistoryItem {
    fn version(&self) -> usize {
        self.version
    }

    fn set_version(&mut self, version: usize) {
        self.version = version;
    }
}

impl SelectionHistoryItem {
    /// Create a history item before the stack assigns its version.
    ///
    /// For example, `new(change)` starts at version zero until the history stack
    /// assigns its version.
    pub(super) const fn new(change: SelectionChange) -> Self {
        Self { version: 0, change }
    }
}

impl Phase0Shell {
    /// Record a selection change unless both snapshots are equal.
    ///
    /// Equal selections produce no history item; for example, repeating an
    /// operation that leaves selection unchanged cannot add an undo step.
    pub(super) fn record_selection_change(&mut self, from: Selection, to: Selection) {
        if from == to {
            return;
        }

        self.selection_history
            .push(SelectionHistoryItem::new(SelectionChange { from, to }));
    }

    /// Restore the previous selection from the latest history group, if present.
    ///
    /// For example, after recording a change from selection A to B, undo restores A.
    pub(super) fn undo_selection(&mut self) {
        let Some(group) = self.selection_history.undo() else {
            return;
        };

        if let Some(last) = group.last() {
            self.state.selection = last.change.from.clone();
        }

        self.select_tool_state = SelectToolState::Idle;
    }

    /// Restore the next selection from the latest history group, if present.
    ///
    /// For example, after undoing a change from selection A to B, redo restores B.
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
