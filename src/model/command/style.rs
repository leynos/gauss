//! Style command helpers.

use crate::model::Document;

use super::inverse::CommandInverse;
use super::types::StyleChange;

pub(super) fn apply_set_style(
    doc: &mut Document,
    changes: &[StyleChange],
    command_name: &'static str,
) -> CommandInverse {
    for change in changes {
        if let Some(shape) = doc.get_mut(change.shape_id) {
            shape.style = change.to.clone();
        }
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

    CommandInverse::RestoreStyles {
        command_name,
        changes: inverse_changes,
    }
}

pub(super) fn apply_restore_styles(doc: &mut Document, changes: &[StyleChange]) {
    for change in changes {
        if let Some(shape) = doc.get_mut(change.shape_id) {
            shape.style = change.to.clone();
        }
    }
}
