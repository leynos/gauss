//! Document history entries for Phase 0.
//!
//! Phase 0 stores document edits as either low-level `DocChange` operations or
//! higher-level Commands with their inverses. This keeps undo/redo behaviour
//! consistent while the migration to Commands is in progress.

use gpui_component::history::HistoryItem;

use crate::model::{Command, CommandInverse, DocChange};

/// A document edit that can be undone and redone.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum DocumentEdit {
    /// Low-level document change captured as `DocOps`.
    Change(DocChange),
    /// Command-based edit with a stored inverse for undo.
    Command(Box<CommandEntry>),
}

/// A command and its inverse used for undo/redo.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CommandEntry {
    pub(super) command: Command,
    pub(super) inverse: CommandInverse,
}

/// History item wrapper for document edits.
#[derive(Clone, Debug)]
pub(super) struct DocumentHistoryItem {
    version: usize,
    edit: DocumentEdit,
}

impl PartialEq for DocumentHistoryItem {
    fn eq(&self, other: &Self) -> bool {
        self.edit == other.edit
    }
}

impl HistoryItem for DocumentHistoryItem {
    fn version(&self) -> usize {
        self.version
    }

    fn set_version(&mut self, version: usize) {
        self.version = version;
    }
}

impl DocumentHistoryItem {
    pub(super) const fn new_doc_change(change: DocChange) -> Self {
        Self {
            version: 0,
            edit: DocumentEdit::Change(change),
        }
    }

    pub(super) fn new_command(command: Command, inverse: CommandInverse) -> Self {
        Self {
            version: 0,
            edit: DocumentEdit::Command(Box::new(CommandEntry { command, inverse })),
        }
    }

    pub(super) fn into_edit(self) -> DocumentEdit {
        self.edit
    }
}
