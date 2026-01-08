//! Style command helpers.

use crate::model::Document;

use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::StyleChange;

/// Apply style changes, returning the inverse command for undo.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
pub(super) fn apply_set_style(
    doc: &mut Document,
    changes: &[StyleChange],
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
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
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
pub(super) fn apply_restore_styles(
    doc: &mut Document,
    changes: &[StyleChange],
) -> Result<(), UserError> {
    for change in changes {
        let shape = doc
            .get_mut(change.shape_id)
            .ok_or(UserError::ShapeNotFound(change.shape_id))?;
        shape.style = change.to.clone();
    }
    Ok(())
}
