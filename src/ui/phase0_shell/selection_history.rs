//! Selection history support for the Phase 0 shell.
//!
//! Phase 0 keeps selection undo/redo separate from document edits. The intent
//! is to validate the dual-history approach early, before adding more complex
//! selection graphs.

use gpui_component::history::HistoryItem;

use crate::model::Selection;

use super::Phase0Shell;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectionChange {
    pub(super) from: Selection,
    pub(super) to: Selection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectionHistoryItem {
    version: usize,
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
    pub(super) const fn new(change: SelectionChange) -> Self {
        Self { version: 0, change }
    }
}

impl Phase0Shell {
    pub(super) fn record_selection_change(&mut self, from: Selection, to: Selection) {
        if from == to {
            return;
        }

        self.selection_history
            .push(SelectionHistoryItem::new(SelectionChange { from, to }));
    }

    pub(super) fn undo_selection(&mut self) {
        let Some(group) = self.selection_history.undo() else {
            return;
        };

        if let Some(first) = group.first() {
            self.selection = first.change.from.clone();
        }

        self.drag_state = None;
    }

    pub(super) fn redo_selection(&mut self) {
        let Some(group) = self.selection_history.redo() else {
            return;
        };

        if let Some(last) = group.last() {
            self.selection = last.change.to.clone();
        }

        self.drag_state = None;
    }
}
