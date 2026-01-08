//! Inverse command definitions and application.

use crate::model::Document;

use super::anchor::{apply_remove_anchor, apply_reopen_path, apply_restore_anchors};
use super::delete_shapes::apply_restore_shapes;
use super::error::UserError;
use super::insert_shape::apply_remove_shape;
use super::movement::{apply_move_anchor_back, apply_move_handle_back, apply_move_shapes_back};
use super::reorder::apply_reverse_reorder;
use super::segment::apply_restore_segment_kinds;
use super::style::apply_restore_styles;
use super::types::{
    AnchorMovement, AnchorRestoration, DeletedShape, HandleMovement, ReorderOp, SegmentChange,
    ShapeInsertion, ShapeMovement, ShapeReplacement, StyleChange,
};

/// The inverse of an applied command, used for undo.
///
/// `CommandInverse` captures everything needed to reverse a command.
/// It is produced by [`crate::model::Command::apply`] and stored in the undo stack.
///
/// # Examples
///
/// ```rust
/// use gauss::model::{CommandInverse, DeletedShape, Document};
///
/// let mut doc = Document::default();
/// let inverse = CommandInverse::RestoreShapes {
///     command_name: "Delete",
///     targets: vec![],
/// };
/// inverse.apply(&mut doc).expect("undo succeeded");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CommandInverse {
    /// Restore deleted shapes to their original positions.
    RestoreShapes {
        /// Name of the original command (for "Undo {name}" menu entries).
        command_name: &'static str,
        /// Shapes to restore, with their original indices.
        targets: Vec<DeletedShape>,
    },

    /// Move shapes back by negated delta.
    MoveShapesBack {
        /// Name of the original command.
        command_name: &'static str,
        /// Shape movements with negated deltas.
        movements: Vec<ShapeMovement>,
    },

    /// Move anchor back to original position.
    MoveAnchorBack {
        /// Name of the original command.
        command_name: &'static str,
        /// Anchor movement data (delta is negated).
        movement: AnchorMovement,
    },

    /// Move handle back to original position.
    MoveHandleBack {
        /// Name of the original command.
        command_name: &'static str,
        /// Handle movement data (from/to swapped).
        movement: HandleMovement,
    },

    /// Restore previous styles.
    RestoreStyles {
        /// Name of the original command.
        command_name: &'static str,
        /// Style changes with from/to swapped.
        changes: Vec<StyleChange>,
    },

    /// Reverse reorder operations.
    ReverseReorder {
        /// Name of the original command.
        command_name: &'static str,
        /// Reorder operations with from/to swapped.
        operations: Vec<ReorderOp>,
    },

    /// Restore previous segment kinds.
    RestoreSegmentKinds {
        /// Name of the original command.
        command_name: &'static str,
        /// Segment changes with old/new swapped.
        changes: Vec<SegmentChange>,
    },

    /// Remove inserted anchor (restore original shape).
    RemoveAnchor {
        /// Name of the original command.
        command_name: &'static str,
        /// Shape replacement with old/new swapped.
        replacement: ShapeReplacement,
    },

    /// Restore anchors that were deleted.
    RestoreAnchors {
        /// Name of the original command.
        command_name: &'static str,
        /// Anchor restorations to apply.
        restorations: Vec<AnchorRestoration>,
    },

    /// Reopen a closed path (restore original shape).
    ReopenPath {
        /// Name of the original command.
        command_name: &'static str,
        /// Shape replacement with old/new swapped.
        replacement: ShapeReplacement,
    },

    /// Remove an inserted shape (restore document state before insertion).
    RemoveShape {
        /// Name of the original command.
        command_name: &'static str,
        /// Shape insertion data (for removal).
        insertion: ShapeInsertion,
    },
}

impl CommandInverse {
    /// Return a human-readable name for this inverse command.
    ///
    /// This is the same name as the original command, for use in
    /// "Undo {name}" menu entries. The name is stored alongside each inverse
    /// variant so it remains available without additional context.
    ///
    /// # Returns
    ///
    /// A static string containing the human-readable command name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::RestoreShapes { command_name, .. }
            | Self::MoveShapesBack { command_name, .. }
            | Self::MoveAnchorBack { command_name, .. }
            | Self::MoveHandleBack { command_name, .. }
            | Self::RestoreStyles { command_name, .. }
            | Self::ReverseReorder { command_name, .. }
            | Self::RestoreSegmentKinds { command_name, .. }
            | Self::RemoveAnchor { command_name, .. }
            | Self::RestoreAnchors { command_name, .. }
            | Self::ReopenPath { command_name, .. }
            | Self::RemoveShape { command_name, .. } => command_name,
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
    /// Returns [`UserError`] if the inverse cannot be applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::{CommandInverse, DeletedShape, Document};
    ///
    /// let mut doc = Document::default();
    /// let inverse = CommandInverse::RestoreShapes {
    ///     command_name: "Delete",
    ///     targets: vec![],
    /// };
    /// inverse.apply(&mut doc).expect("undo succeeded");
    /// ```
    pub fn apply(&self, doc: &mut Document) -> Result<(), UserError> {
        match self {
            Self::RestoreShapes { targets, .. } => apply_restore_shapes(doc, targets),
            Self::MoveShapesBack { movements, .. } => apply_move_shapes_back(doc, movements),
            Self::MoveAnchorBack { movement, .. } => apply_move_anchor_back(doc, movement),
            Self::MoveHandleBack { movement, .. } => apply_move_handle_back(doc, movement),
            Self::RestoreStyles { changes, .. } => {
                apply_restore_styles(doc, changes);
                Ok(())
            }
            Self::ReverseReorder { operations, .. } => {
                apply_reverse_reorder(doc, operations);
                Ok(())
            }
            Self::RestoreSegmentKinds { changes, .. } => {
                apply_restore_segment_kinds(doc, changes);
                Ok(())
            }
            Self::RemoveAnchor { replacement, .. } => {
                apply_remove_anchor(doc, replacement);
                Ok(())
            }
            Self::RestoreAnchors { restorations, .. } => {
                apply_restore_anchors(doc, restorations);
                Ok(())
            }
            Self::ReopenPath { replacement, .. } => {
                apply_reopen_path(doc, replacement);
                Ok(())
            }
            Self::RemoveShape { insertion, .. } => apply_remove_shape(doc, insertion),
        }
    }
}
