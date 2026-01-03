//! Command definitions and application.

use crate::model::Document;

use super::anchor::{apply_close_path, apply_delete_anchors, apply_insert_anchor};
use super::delete_shapes::apply_delete_shapes;
use super::error::UserError;
use super::inverse::CommandInverse;
use super::movement::{apply_move_anchor, apply_move_handle, apply_move_shapes};
use super::reorder::apply_reorder;
use super::segment::apply_set_segment_kind;
use super::style::apply_set_style;
use super::types::{
    AnchorDeletion, AnchorMovement, DeletedShape, HandleMovement, ReorderOp, SegmentChange,
    ShapeMovement, ShapeReplacement, StyleChange,
};

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
/// ```rust
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

    /// Move shapes by a delta.
    MoveShapes {
        /// Shape movements to apply.
        movements: Vec<ShapeMovement>,
    },

    /// Move an anchor point (including its handles).
    MoveAnchor {
        /// Anchor movement data.
        movement: AnchorMovement,
    },

    /// Move a single handle.
    MoveHandle {
        /// Handle movement data.
        movement: HandleMovement,
    },

    /// Set style on shapes.
    SetStyle {
        /// Style changes to apply.
        changes: Vec<StyleChange>,
    },

    /// Reorder shapes (raise/lower).
    Reorder {
        /// Reorder operations to apply.
        operations: Vec<ReorderOp>,
    },

    /// Toggle segment kind (Line/Cubic).
    SetSegmentKind {
        /// Segment changes to apply.
        changes: Vec<SegmentChange>,
    },

    /// Insert an anchor into a path.
    InsertAnchor {
        /// Shape replacement data.
        replacement: ShapeReplacement,
    },

    /// Delete selected anchors.
    DeleteAnchors {
        /// Anchor deletions to apply.
        deletions: Vec<AnchorDeletion>,
    },

    /// Close an open path.
    ClosePath {
        /// Shape replacement data.
        replacement: ShapeReplacement,
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
    /// ```rust
    /// use gauss::model::Command;
    ///
    /// let cmd = Command::DeleteShapes { targets: vec![] };
    /// assert_eq!(cmd.name(), "Delete");
    /// ```
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DeleteShapes { .. } => "Delete",
            Self::MoveShapes { .. } => "Move",
            Self::MoveAnchor { .. } => "Move Anchor",
            Self::MoveHandle { .. } => "Move Handle",
            Self::SetStyle { .. } => "Set Style",
            Self::Reorder { .. } => "Reorder",
            Self::SetSegmentKind { .. } => "Toggle Segment",
            Self::InsertAnchor { .. } => "Insert Anchor",
            Self::DeleteAnchors { .. } => "Delete Anchors",
            Self::ClosePath { .. } => "Close Path",
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
    /// Returns [`UserError`] if the command cannot be executed (e.g.,
    /// referenced shapes do not exist). In practice, commands prepared via
    /// [`crate::model::prepare_command`] should not fail during application.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::{Command, CommandInverse, DeletedShape, Document};
    ///
    /// let mut doc = Document::default();
    /// let cmd = Command::DeleteShapes { targets: vec![] };
    /// let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    /// ```
    pub fn apply(&self, doc: &mut Document) -> Result<CommandInverse, UserError> {
        let command_name = self.name();
        match self {
            Self::DeleteShapes { targets } => Ok(apply_delete_shapes(doc, targets, command_name)),
            Self::MoveShapes { movements } => Ok(apply_move_shapes(doc, movements, command_name)),
            Self::MoveAnchor { movement } => Ok(apply_move_anchor(doc, movement, command_name)),
            Self::MoveHandle { movement } => Ok(apply_move_handle(doc, movement, command_name)),
            Self::SetStyle { changes } => Ok(apply_set_style(doc, changes, command_name)),
            Self::Reorder { operations } => Ok(apply_reorder(doc, operations, command_name)),
            Self::SetSegmentKind { changes } => {
                Ok(apply_set_segment_kind(doc, changes, command_name))
            }
            Self::InsertAnchor { replacement } => {
                Ok(apply_insert_anchor(doc, replacement, command_name))
            }
            Self::DeleteAnchors { deletions } => {
                Ok(apply_delete_anchors(doc, deletions, command_name))
            }
            Self::ClosePath { replacement } => Ok(apply_close_path(doc, replacement, command_name)),
        }
    }
}
