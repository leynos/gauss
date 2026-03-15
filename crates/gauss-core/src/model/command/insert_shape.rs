//! Insert shape command implementation.
//!
//! This module implements shape insertion within the command/undo system.
//! It provides the forward operation ([`apply_insert_shape`]) that adds a
//! shape to the document, and the inverse operation ([`apply_remove_shape`])
//! that removes it during undo.
//!
//! The [`ShapeInsertion`] type captures the insertion index and shape data
//! needed for both directions: applying the insertion and reversing it on
//! undo. This symmetry between insertion and removal ensures consistent
//! undo/redo behaviour.
//!
//! # Index validity
//!
//! The stored insertion index in [`ShapeInsertion`] is only valid when
//! commands are applied in strict undo/redo sequence order. The command
//! history system guarantees this ordering: undo operations are applied in
//! reverse chronological order, and redo operations replay in forward order.
//!
//! Intervening edits between undo and redo (i.e., forking the history by
//! making new changes after an undo) invalidate the redo stack entirely,
//! so stale indices are never applied. This design assumes the history
//! manager clears the redo stack when new commands are executed.
//!
//! # Integration
//!
//! - [`Command::InsertShape`](super::Command::InsertShape) uses
//!   [`apply_insert_shape`] to insert shapes and produce a
//!   [`CommandInverse::RemoveShape`](super::CommandInverse::RemoveShape).
//! - [`CommandInverse::RemoveShape`](super::CommandInverse::RemoveShape)
//!   uses [`apply_remove_shape`] to restore the document state before
//!   insertion.

use crate::model::Document;

use super::CommandInverse;
use super::error::UserError;
use super::types::ShapeInsertion;

/// Apply the `InsertShape` command, returning the inverse for undo.
///
/// # Errors
///
/// Returns `UserError::InvalidOperation` if the insertion index is out of range.
pub fn apply_insert_shape(
    doc: &mut Document,
    insertion: &ShapeInsertion,
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    debug_assert!(
        insertion.index <= doc.len(),
        "apply_insert_shape: insertion index {} out of range (len = {})",
        insertion.index,
        doc.len()
    );

    if insertion.index > doc.len() {
        return Err(UserError::InvalidOperation(format!(
            "shape insertion index {} out of range (len = {})",
            insertion.index,
            doc.len()
        )));
    }

    let _shape_id = doc
        .insert_shape(insertion.index, insertion.shape.clone())
        .ok_or_else(|| {
            UserError::InvalidOperation(format!(
                "insert_shape failed for index {} (len = {})",
                insertion.index,
                doc.len()
            ))
        })?;

    Ok(CommandInverse::RemoveShape {
        command_name,
        insertion: insertion.clone(),
    })
}

/// Apply the `RemoveShape` inverse command.
///
/// # Errors
///
/// Returns `UserError::InvalidOperation` if the removal index is out of range.
pub fn apply_remove_shape(doc: &mut Document, insertion: &ShapeInsertion) -> Result<(), UserError> {
    debug_assert!(
        insertion.index < doc.len(),
        "apply_remove_shape: insertion index {} out of range (len = {})",
        insertion.index,
        doc.len()
    );

    if insertion.index >= doc.len() {
        return Err(UserError::InvalidOperation(format!(
            "shape removal index {} out of range (len = {})",
            insertion.index,
            doc.len()
        )));
    }

    doc.remove_shape(insertion.index).ok_or_else(|| {
        UserError::InvalidOperation(format!(
            "remove_shape failed for index {} (len = {})",
            insertion.index,
            doc.len()
        ))
    })?;
    Ok(())
}
