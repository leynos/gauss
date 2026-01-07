//! Document history entries for Phase 0.
//!
//! Document edits are stored as Commands with their inverses, enabling
//! consistent undo/redo behaviour and future session replay.

use gpui_component::history::HistoryItem;

use crate::model::{Command, CommandInverse};

/// A document edit that can be undone and redone.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DocumentEdit {
    /// Command-based edit with a stored inverse for undo.
    pub(super) entry: Box<CommandEntry>,
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
    pub(super) fn new_command(command: Command, inverse: CommandInverse) -> Self {
        Self {
            version: 0,
            edit: DocumentEdit {
                entry: Box::new(CommandEntry { command, inverse }),
            },
        }
    }

    pub(super) fn into_entry(self) -> CommandEntry {
        *self.edit.entry
    }
}
