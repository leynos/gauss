//! Undoable Commands for the Gauss editor.
//!
//! Commands are concrete, undoable state changes. They sit between Actions
//! (user intent) and DocOps (atomic mutations). Commands capture pre-
//! conditions, required context, and sufficient data for undo.
//!
//! Commands are GPUI-independent for testability and scripting.

use crate::model::{Shape, ShapeId};

/// User-facing errors that can occur during command preparation or execution.
///
/// These errors represent semantic issues that should be presented to users
/// (e.g., via UI messages, disabled menu items, or accessibility feedback).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum UserError {
    /// The command requires a non-empty selection, but nothing is selected.
    #[error("No selection")]
    EmptySelection,

    /// A referenced shape does not exist in the document.
    #[error("Shape not found")]
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
