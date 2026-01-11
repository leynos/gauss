//! Style command helpers.

use crate::model::Document;

use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::StyleChange;

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
    // Pre-validate: ensure all shapes exist before applying any changes.
    for change in changes {
        if doc.find_index(change.shape_id).is_none() {
            return Err(UserError::ShapeNotFound(change.shape_id));
        }
    }

    // Apply changes (safe now that all shapes are validated).
    for change in changes {
        let shape = doc
            .get_mut(change.shape_id)
            .ok_or(UserError::ShapeNotFound(change.shape_id))?;
        shape.style = change.to.clone();
    }

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
    // Pre-validate: ensure all shapes exist before applying any changes.
    for change in changes {
        if doc.find_index(change.shape_id).is_none() {
            return Err(UserError::ShapeNotFound(change.shape_id));
        }
    }

    // Apply changes (safe now that all shapes are validated).
    for change in changes {
        let shape = doc
            .get_mut(change.shape_id)
            .ok_or(UserError::ShapeNotFound(change.shape_id))?;
        shape.style = change.to.clone();
    }
    Ok(())
}
