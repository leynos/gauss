//! Style command helpers.

use super::{error::UserError, inverse::CommandInverse, types::StyleChange};
use crate::model::Document;

/// Apply style changes, returning the inverse command for undo.
///
/// Pre-validates all shape IDs before applying any changes to ensure atomicity:
/// either all changes succeed or none are applied.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
pub(super) fn apply_set_style(
    doc: &mut Document,
    changes: &[StyleChange],
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    validate_shapes_exist(doc, changes)?;
    apply_style_changes(doc, changes)?;

    // Create inverse with swapped from/to
    let inverse_changes = changes
        .iter()
        .map(|c| StyleChange {
            shape_id: c.shape_id,
            from: c.to.clone(),
            to: c.from.clone(),
        })
        .collect();

    Ok(CommandInverse::RestoreStyles {
        command_name,
        changes: inverse_changes,
    })
}

/// Apply the inverse of style changes (for undo).
///
/// Pre-validates all shape IDs before applying any changes to ensure atomicity:
/// either all changes succeed or none are applied.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
pub(super) fn apply_restore_styles(
    doc: &mut Document,
    changes: &[StyleChange],
) -> Result<(), UserError> {
    validate_shapes_exist(doc, changes)?;
    apply_style_changes(doc, changes)
}

/// Ensure every referenced shape exists before any change is applied.
///
/// Validation runs separately from application so a missing shape leaves the
/// document untouched, preserving the atomicity of the command pair.
fn validate_shapes_exist(doc: &Document, changes: &[StyleChange]) -> Result<(), UserError> {
    changes.iter().try_for_each(|change| {
        if doc.find_index(change.shape_id).is_some() {
            Ok(())
        } else {
            Err(UserError::ShapeNotFound(change.shape_id))
        }
    })
}

/// Write each style change into its shape.
///
/// Callers must run [`validate_shapes_exist`] first; the shape lookup here is
/// infallible in practice but still reports a missing shape rather than
/// silently skipping the change.
fn apply_style_changes(doc: &mut Document, changes: &[StyleChange]) -> Result<(), UserError> {
    for change in changes {
        let shape = doc
            .shape_mut(change.shape_id)
            .ok_or(UserError::ShapeNotFound(change.shape_id))?;
        shape.style = change.to.clone();
    }
    Ok(())
}
