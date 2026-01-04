//! Delete shapes command implementation.

use crate::model::{Document, Selection, ShapeId};

use super::error::UserError;
use super::types::DeletedShape;
use super::{Command, CommandInverse};

/// Prepare a `DeleteShapes` command from the current selection.
///
/// # Errors
///
/// Returns [`UserError::EmptySelection`] if nothing is selected or only
/// anchors/handles are selected (no whole shapes).
pub fn prepare_delete_selection(
    doc: &Document,
    selection: &Selection,
) -> Result<Command, UserError> {
    if selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    // Collect selected shape IDs
    let shape_ids: Vec<ShapeId> = selection.selected_shapes().collect();

    if shape_ids.is_empty() {
        // Selection contains only anchors/handles/segments, no whole shapes
        return Err(UserError::EmptySelection);
    }

    // Build DeletedShape entries with indices and data
    let mut targets = Vec::with_capacity(shape_ids.len());
    for &id in &shape_ids {
        let Some(index) = doc.find_index(id) else {
            return Err(UserError::ShapeNotFound(id));
        };
        // find_index guarantees valid index; if violated, treat as shape not found
        // (defensive: avoids panic in production while preserving error semantics)
        let Some(shape) = doc.shapes.get(index).cloned() else {
            return Err(UserError::ShapeNotFound(id));
        };
        targets.push(DeletedShape { index, shape });
    }

    Ok(Command::DeleteShapes { targets })
}

/// Apply the `DeleteShapes` command, returning the inverse for undo.
pub fn apply_delete_shapes(
    doc: &mut Document,
    targets: &[DeletedShape],
    command_name: &'static str,
) -> CommandInverse {
    // Remove shapes in reverse index order to preserve indices during removal
    let mut sorted_indices: Vec<usize> = targets.iter().map(|t| t.index).collect();
    sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

    for &index in &sorted_indices {
        debug_assert!(
            index < doc.shapes.len(),
            "apply_delete_shapes: index {index} out of range (len = {}), \
             this likely indicates document corruption or a logic bug",
            doc.shapes.len()
        );
        if index < doc.shapes.len() {
            doc.shapes.remove(index);
        }
    }

    CommandInverse::RestoreShapes {
        command_name,
        targets: targets.to_vec(),
    }
}

/// Apply the `RestoreShapes` inverse command.
pub fn apply_restore_shapes(doc: &mut Document, targets: &[DeletedShape]) {
    // Insert shapes in forward index order to preserve indices during insertion
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by_key(|t| t.index);

    for target in sorted_targets {
        debug_assert!(
            target.index <= doc.shapes.len(),
            "apply_restore_shapes: target index {} out of range (len = {}), \
             this likely indicates document corruption or a logic bug",
            target.index,
            doc.shapes.len()
        );
        if target.index <= doc.shapes.len() {
            doc.shapes.insert(target.index, target.shape);
        }
    }
}
