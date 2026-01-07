//! Insert shape command implementation.

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
        insertion.index <= doc.shapes.len(),
        "apply_insert_shape: insertion index {} out of range (len = {})",
        insertion.index,
        doc.shapes.len()
    );

    if insertion.index > doc.shapes.len() {
        return Err(UserError::InvalidOperation(
            "shape insertion index out of range",
        ));
    }

    doc.shapes.insert(insertion.index, insertion.shape.clone());

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
        insertion.index < doc.shapes.len(),
        "apply_remove_shape: insertion index {} out of range (len = {})",
        insertion.index,
        doc.shapes.len()
    );

    if insertion.index >= doc.shapes.len() {
        return Err(UserError::InvalidOperation(
            "shape removal index out of range",
        ));
    }

    doc.shapes.remove(insertion.index);
    Ok(())
}
