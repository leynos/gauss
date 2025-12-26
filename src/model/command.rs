//! Undoable Commands for the Gauss editor.
//!
//! Commands are concrete, undoable state changes. They sit between Actions
//! (user intent) and `DocOps` (atomic mutations). Commands capture pre-
//! conditions, required context, and sufficient data for undo.
//!
//! Commands are GPUI-independent for testability and scripting.
//!
//! # Design
//!
//! Commands are implemented as an enum rather than a trait for several reasons:
//!
//! - **Exhaustive matching**: All command variants can be matched exhaustively,
//!   making dispatch tables complete and verifiable at compile time.
//! - **Serialization**: Enums are trivially serializable, enabling future macro
//!   recording and playback (see roadmap §0.1.2).
//! - **Simplicity**: No type erasure or dynamic dispatch complexity.
//! - **Consistency**: Matches the Action enum design from task 0.1.1.
//!
//! # Relationship to Actions and `DocOps`
//!
//! ```text
//! Action (user intent)       e.g., DeleteSelection
//!    │
//!    ▼  prepare_command()
//! Command (undoable mutation) e.g., DeleteShapes { targets: [...] }
//!    │
//!    ▼  apply()
//! Document mutation          Direct shape removal with inverse capture
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use gauss::model::{
//!     Action, Command, Document, Selection, SelItem, Shape, prepare_command,
//! };
//!
//! // Prepare a command from an action
//! let doc = Document::default();
//! let selection = Selection::default();
//! let result = prepare_command(Action::DeleteSelection, &doc, &selection);
//!
//! // Empty selection produces an error
//! assert!(result.is_err());
//! ```

use crate::model::{Action, Document, Selection, Shape, ShapeId};

/// Errors that can occur during command preparation or execution.
///
/// These are semantic errors that the caller can inspect and handle
/// appropriately (e.g., disable a menu item, show an error message).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The command requires a non-empty selection, but nothing is selected.
    #[error("command requires a selection, but nothing is selected")]
    EmptySelection,

    /// A referenced shape does not exist in the document.
    #[error("shape {0:?} not found in document")]
    ShapeNotFound(ShapeId),

    /// An action that does not produce a command was passed to `prepare_command`.
    ///
    /// This indicates a dispatcher bug: Editor actions (Undo, Redo, tool
    /// activations, etc.) should be routed directly, not via `prepare_command`.
    #[error("action {0:?} does not produce a command")]
    NotACommand(Action),
}

/// A shape that was deleted, with data needed for restoration.
///
/// Captures both the original index (for reinsertion) and the full shape
/// data (for restoration on undo).
#[derive(Clone, Debug, PartialEq)]
pub struct DeletedShape {
    /// Original index in the document's shape list.
    pub index: usize,
    /// The deleted shape data.
    pub shape: Shape,
}

/// Concrete, undoable state changes.
///
/// Commands are the unit of undo/redo. Each command captures sufficient
/// data to apply and reverse the operation.
///
/// # Variants
///
/// This enum uses `#[non_exhaustive]` to allow adding new command variants
/// in future versions without breaking downstream code.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{Command, DeletedShape};
///
/// // Commands can be matched exhaustively within this crate
/// let cmd = Command::DeleteShapes { targets: vec![] };
/// let name = cmd.name();
/// assert_eq!(name, "Delete");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Command {
    /// Delete the specified shapes from the document.
    DeleteShapes {
        /// Shapes to delete, with their indices and data for undo.
        targets: Vec<DeletedShape>,
    },
}

impl Command {
    /// Return a human-readable name for this command.
    ///
    /// This name is suitable for:
    ///
    /// - Undo/redo menu entries ("Undo Delete")
    /// - Accessibility labels
    /// - Scripting API documentation
    ///
    /// Note: These names will be replaced with localized strings when the
    /// i18n scaffolding (task 0.7) is implemented.
    ///
    /// # Returns
    ///
    /// A static string containing the human-readable command name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::Command;
    ///
    /// let cmd = Command::DeleteShapes { targets: vec![] };
    /// assert_eq!(cmd.name(), "Delete");
    /// ```
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DeleteShapes { .. } => "Delete",
        }
    }

    /// Apply the command to the document, returning the inverse for undo.
    ///
    /// # Parameters
    ///
    /// - `doc`: The document to mutate.
    ///
    /// # Returns
    ///
    /// A [`CommandInverse`] that can be applied to restore the previous state.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] if the command cannot be executed (e.g.,
    /// referenced shapes do not exist). In practice, commands prepared via
    /// [`prepare_command`] should not fail during application.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::{Command, CommandInverse, DeletedShape, Document};
    ///
    /// let mut doc = Document::default();
    /// let cmd = Command::DeleteShapes { targets: vec![] };
    /// let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    /// ```
    pub fn apply(&self, doc: &mut Document) -> Result<CommandInverse, CommandError> {
        match self {
            Self::DeleteShapes { targets } => Ok(apply_delete_shapes(doc, targets)),
        }
    }
}

/// The inverse of an applied command, used for undo.
///
/// `CommandInverse` captures everything needed to reverse a command.
/// It is produced by [`Command::apply`] and stored in the undo stack.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{CommandInverse, DeletedShape, Document};
///
/// let mut doc = Document::default();
/// let inverse = CommandInverse::RestoreShapes { targets: vec![] };
/// inverse.apply(&mut doc).expect("undo succeeded");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CommandInverse {
    /// Restore deleted shapes to their original positions.
    RestoreShapes {
        /// Shapes to restore, with their original indices.
        targets: Vec<DeletedShape>,
    },
}

impl CommandInverse {
    /// Return a human-readable name for this inverse command.
    ///
    /// This is the same name as the original command, for use in
    /// "Undo {name}" menu entries.
    ///
    /// # Returns
    ///
    /// A static string containing the human-readable command name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::RestoreShapes { .. } => "Delete",
        }
    }

    /// Apply the inverse command to restore previous state.
    ///
    /// # Parameters
    ///
    /// - `doc`: The document to mutate.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] if the inverse cannot be applied.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::{CommandInverse, DeletedShape, Document};
    ///
    /// let mut doc = Document::default();
    /// let inverse = CommandInverse::RestoreShapes { targets: vec![] };
    /// inverse.apply(&mut doc).expect("undo succeeded");
    /// ```
    pub fn apply(&self, doc: &mut Document) -> Result<(), CommandError> {
        match self {
            Self::RestoreShapes { targets } => {
                apply_restore_shapes(doc, targets);
                Ok(())
            }
        }
    }
}

/// Prepare a command from an action and current editor state.
///
/// This function bridges user intent (Action) to concrete command (Command).
/// It captures required context (selection, document state) at the moment
/// the action is invoked.
///
/// # Parameters
///
/// - `action`: The user action to convert.
/// - `doc`: The current document state (for capturing shape data).
/// - `selection`: The current selection (for determining targets).
///
/// # Returns
///
/// A [`Command`] ready for execution, or an error if the action cannot
/// produce a valid command.
///
/// # Errors
///
/// Returns [`CommandError`] if the action cannot produce a valid command:
///
/// - [`CommandError::EmptySelection`]: Action requires selection but none exists.
/// - [`CommandError::ShapeNotFound`]: Selected shape not in document.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{Action, Document, Selection, prepare_command};
///
/// let doc = Document::default();
/// let selection = Selection::default();
///
/// // Empty selection produces an error
/// let result = prepare_command(Action::DeleteSelection, &doc, &selection);
/// assert!(result.is_err());
/// ```
pub fn prepare_command(
    action: Action,
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    match action {
        Action::DeleteSelection => prepare_delete_selection(doc, selection),
        // Editor actions do not produce commands; this is a dispatcher bug
        Action::SelectAll
        | Action::DeselectAll
        | Action::ActivatePenTool
        | Action::ActivateSelectTool
        | Action::Undo
        | Action::Redo => Err(CommandError::NotACommand(action)),
    }
}

fn prepare_delete_selection(
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    if selection.is_empty() {
        return Err(CommandError::EmptySelection);
    }

    // Collect selected shape IDs
    let shape_ids: Vec<ShapeId> = selection.selected_shapes().collect();

    if shape_ids.is_empty() {
        // Selection contains only anchors/handles/segments, no whole shapes
        return Err(CommandError::EmptySelection);
    }

    // Build DeletedShape entries with indices and data
    let mut targets = Vec::with_capacity(shape_ids.len());
    for &id in &shape_ids {
        let Some(index) = doc.find_index(id) else {
            return Err(CommandError::ShapeNotFound(id));
        };
        // find_index returned Some, so index is valid; use .get() for clippy
        let Some(shape) = doc.shapes.get(index).cloned() else {
            return Err(CommandError::ShapeNotFound(id));
        };
        targets.push(DeletedShape { index, shape });
    }

    Ok(Command::DeleteShapes { targets })
}

fn apply_delete_shapes(doc: &mut Document, targets: &[DeletedShape]) -> CommandInverse {
    // Remove shapes in reverse index order to preserve indices during removal
    let mut sorted_indices: Vec<usize> = targets.iter().map(|t| t.index).collect();
    sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

    for &index in &sorted_indices {
        debug_assert!(
            index < doc.shapes.len(),
            "apply_delete_shapes: index {index} out of range (len = {}), \
             this likely indicates document corruption or a logic bug",
            doc.shapes.len()
        );
        if index < doc.shapes.len() {
            doc.shapes.remove(index);
        }
    }

    CommandInverse::RestoreShapes {
        targets: targets.to_vec(),
    }
}

fn apply_restore_shapes(doc: &mut Document, targets: &[DeletedShape]) {
    // Insert shapes in forward index order to preserve indices during insertion
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by_key(|t| t.index);

    for target in sorted_targets {
        debug_assert!(
            target.index <= doc.shapes.len(),
            "apply_restore_shapes: target index {} out of range (len = {}), \
             this likely indicates document corruption or a logic bug",
            target.index,
            doc.shapes.len()
        );
        if target.index <= doc.shapes.len() {
            doc.shapes.insert(target.index, target.shape);
        }
    }
}
