//! Model-layer document undo/redo history.
//!
//! This module provides GPUI-independent undo/redo backed by the `undo_2`
//! crate.  [`DocumentUndoHistory`] wraps `undo_2::Commands` and stores
//! paired [`Command`] / [`CommandInverse`] entries so that both forward
//! replay and reversal are possible without recomputing inverses.
//!
//! `undo_2` implements *historical* undo — commands are never discarded
//! after a branch edit, making the full edit history navigable.

use undo_2::Action;

use super::command::Command;
use super::command::CommandInverse;
use super::document::Document;

#[cfg(test)]
mod tests;

/// Default maximum number of commands retained in the undo history.
///
/// Generous for an illustration editor; bounds memory in long sessions.
const DEFAULT_MAX_DEPTH: usize = 500;

/// A paired command and its inverse, stored in the undo history.
///
/// Both sides are retained so that `undo_2` can instruct the adapter to
/// apply either the forward or reverse operation without recomputing.
#[derive(Clone, Debug, PartialEq)]
struct HistoryEntry {
    command: Command,
    inverse: CommandInverse,
}

/// GPUI-independent document undo/redo history.
///
/// Wraps [`undo_2::Commands`] to provide a high-level `record` / `undo` /
/// `redo` API over [`Command`] and [`CommandInverse`] pairs.
///
/// # Examples
///
/// ```rust
/// use gauss::model::history::DocumentUndoHistory;
/// use gauss::model::{Command, Document};
///
/// let mut doc = Document::default();
/// let mut history = DocumentUndoHistory::new();
///
/// // After recording a command, undo is available.
/// let cmd = Command::DeleteShapes { targets: vec![] };
/// let inverse = cmd.apply(&mut doc).expect("apply succeeds");
/// history.record(cmd, inverse);
/// assert!(history.can_undo());
/// ```
pub struct DocumentUndoHistory {
    commands: undo_2::Commands<HistoryEntry>,
    max_depth: usize,
}

impl DocumentUndoHistory {
    /// Create an empty history with no recorded commands.
    ///
    /// Uses the default depth limit of 500.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: undo_2::Commands::new(),
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// Create an empty history with a custom depth limit.
    ///
    /// After each `record()` call, entries beyond `max_depth` are
    /// pruned via `keep_last()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::history::DocumentUndoHistory;
    ///
    /// let history = DocumentUndoHistory::with_max_depth(100);
    /// assert_eq!(history.max_depth(), 100);
    /// ```
    #[must_use]
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            commands: undo_2::Commands::new(),
            max_depth,
        }
    }

    /// Return the configured maximum history depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Record a command that has already been applied to the document.
    ///
    /// The caller is responsible for applying `command` to the document
    /// *before* calling this method and passing the resulting `inverse`.
    pub fn record(&mut self, command: Command, inverse: CommandInverse) {
        self.commands.push(HistoryEntry { command, inverse });
        self.commands.keep_last(self.max_depth);
    }

    /// Undo the most recent action, mutating the document accordingly.
    ///
    /// When the history is empty or fully undone this is a no-op returning
    /// `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns a description if any individual command or inverse
    /// application fails.
    pub fn undo(&mut self, doc: &mut Document) -> Result<(), String> {
        let actions: Vec<_> = self.commands.undo().collect();
        Self::apply_actions(doc, &actions, "Undo")
    }

    /// Redo the most recently undone action, mutating the document.
    ///
    /// When there is nothing to redo this is a no-op returning `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns a description if any individual command or inverse
    /// application fails.
    pub fn redo(&mut self, doc: &mut Document) -> Result<(), String> {
        let actions: Vec<_> = self.commands.redo().collect();
        Self::apply_actions(doc, &actions, "Redo")
    }

    /// Discard all recorded commands, resetting the history to empty.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Return whether there is at least one action that can be undone.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.commands.iter_realized().next().is_some()
    }

    /// Return whether there is at least one action that can be redone.
    ///
    /// `undo_2` tracks redo availability by appending an internal
    /// `Undo(n)` item when undo is called; `Commands::redo` only
    /// produces actions when such an item is present.
    /// `Commands::is_undoing()` checks exactly that condition, so it
    /// is the correct predicate for "can redo".
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.commands.is_undoing()
    }

    /// Return the number of realised (non-undone) history entries.
    ///
    /// Undone entries are not counted, so this reflects the user-visible
    /// "undo stack depth" — the number of times `undo()` can be called
    /// before the history is exhausted.
    ///
    /// # Performance
    ///
    /// This method walks the internal `iter_realized()` iterator on each
    /// call, so it runs in *O(n)* where *n* is the number of history
    /// entries.  It is intended for assertions in tests and infrequent UI
    /// queries (e.g. menu-item enable state), not for hot paths.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::history::DocumentUndoHistory;
    ///
    /// let history = DocumentUndoHistory::new();
    /// assert_eq!(history.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.iter_realized().count()
    }

    /// Return whether the history contains no realised entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::history::DocumentUndoHistory;
    ///
    /// let history = DocumentUndoHistory::new();
    /// assert!(history.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Apply a collected list of `(Action, &HistoryEntry)` pairs to the
    /// document.
    ///
    /// All actions in the batch are applied even when one fails, because
    /// `undo_2` has already advanced its cursor over the entire group.
    /// Skipping remaining actions would leave the document out of sync
    /// with the history position.  Only the first error is surfaced.
    fn apply_actions(
        doc: &mut Document,
        actions: &[(Action, &HistoryEntry)],
        label: &str,
    ) -> Result<(), String> {
        let mut first_error: Option<String> = None;
        for (action, entry) in actions {
            if let Err(e) = Self::apply_single(doc, *action, entry, label) {
                first_error.get_or_insert(e);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    /// Execute one `(Action, HistoryEntry)` step against the document.
    fn apply_single(
        doc: &mut Document,
        action: Action,
        entry: &HistoryEntry,
        label: &str,
    ) -> Result<(), String> {
        match action {
            Action::Undo => entry
                .inverse
                .apply(doc)
                .map_err(|e| format!("{label} failed for '{}': {e}", entry.inverse.name())),
            Action::Do => {
                // The new inverse is intentionally discarded — the stored
                // inverse remains correct for future undo cycles.
                entry
                    .command
                    .apply(doc)
                    .map(|_inverse| ())
                    .map_err(|e| format!("{label} failed for '{}': {e}", entry.command.name()))
            }
        }
    }
}

impl Default for DocumentUndoHistory {
    fn default() -> Self {
        Self::new()
    }
}
