//! Undoable Commands for the Gauss editor.
//!
//! Commands are concrete, undoable state changes. They sit between Actions
//! (user intent) and DocOps (atomic mutations). Commands capture pre-
//! conditions, required context, and sufficient data for undo.
//!
//! Commands are GPUI-independent for testability and scripting.

use crate::model::{Document, DocChange, DocOp, Selection, Shape, ShapeId};

/// Errors that can occur during command execution.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The command requires a non-empty selection, but nothing is selected.
    #[error("command requires a selection, but nothing is selected")]
    EmptySelection,

    /// A referenced shape does not exist in the document.
    #[error("shape {0:?} not found in document")]
    ShapeNotFound(ShapeId),
}

/// A shape that was deleted, with data needed for restoration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Command {
    /// Delete the specified shapes from the document.
    DeleteShapes {
        /// Shapes to delete, with their indices and data for undo.
        targets: Vec<DeletedShape>,
    },
}
