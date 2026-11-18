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
    validate_style_changes(doc, changes)?;
    apply_style_changes(doc, changes)?;
    Ok(CommandInverse::RestoreStyles {
        command_name,
        changes: inverted_style_changes(changes),
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
    validate_style_changes(doc, changes)?;
    apply_style_changes(doc, changes)
}

/// Requires every referenced shape to exist.
///
/// Both style commands validate the whole batch first, so a failed batch
/// leaves the document untouched.
fn validate_style_changes(doc: &Document, changes: &[StyleChange]) -> Result<(), UserError> {
    for change in changes {
        if doc.find_index(change.shape_id).is_none() {
            return Err(UserError::ShapeNotFound(change.shape_id));
        }
    }
    Ok(())
}

/// Applies every change; call [`validate_style_changes`] first.
fn apply_style_changes(doc: &mut Document, changes: &[StyleChange]) -> Result<(), UserError> {
    for change in changes {
        let shape = doc
            .shape_mut(change.shape_id)
            .ok_or(UserError::ShapeNotFound(change.shape_id))?;
        shape.style = change.to.clone();
    }
    Ok(())
}

/// Builds the inverse changes, swapping `from` and `to`.
fn inverted_style_changes(changes: &[StyleChange]) -> Vec<StyleChange> {
    changes
        .iter()
        .map(|c| StyleChange {
            shape_id: c.shape_id,
            from: c.to.clone(),
            to: c.from.clone(),
        })
        .collect()
}
