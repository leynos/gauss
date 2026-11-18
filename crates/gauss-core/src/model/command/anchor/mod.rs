//! Anchor and path command helpers.

use std::collections::HashMap;

use super::{
    Command,
    error::UserError,
    inverse::CommandInverse,
    types::{
        AnchorDeletion,
        AnchorDeletionResult,
        AnchorRestoration,
        AnchorRestorationKind,
        ShapeReplacement,
    },
};
use crate::model::{Document, EngineState, PathGeom, SelItem, Shape, ShapeId};

mod insert;

pub(super) use insert::prepare_insert_anchor_on_segment;

/// Group selected anchors by shape and remove shapes left with fewer than two.
/// Deleting two anchors from a three-anchor shape therefore schedules its removal.
pub(super) fn prepare_delete_selected_anchors(state: &EngineState) -> Result<Command, UserError> {
    let anchors_by_shape = selected_anchor_indices(state)?;
    let deletions = anchors_by_shape
        .into_iter()
        .map(|(shape_id, anchor_indices)| {
            anchor_deletion_for_shape(state, shape_id, anchor_indices)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Command::DeleteAnchors { deletions })
}

/// Collect the selected anchor indices, grouped by their owning shape.
fn selected_anchor_indices(state: &EngineState) -> Result<HashMap<ShapeId, Vec<usize>>, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    let mut anchors_by_shape: HashMap<ShapeId, Vec<usize>> = HashMap::new();
    for item in &state.selection.items {
        if let SelItem::Anchor { shape, anchor } = item {
            anchors_by_shape.entry(*shape).or_default().push(*anchor);
        }
    }

    if anchors_by_shape.is_empty() {
        return Err(UserError::InvalidOperation("No anchors selected".into()));
    }

    Ok(anchors_by_shape)
}

/// Build the deletion record for one shape's selected anchors.
fn anchor_deletion_for_shape(
    state: &EngineState,
    shape_id: ShapeId,
    mut anchor_indices: Vec<usize>,
) -> Result<AnchorDeletion, UserError> {
    let shape_index = state
        .document
        .find_index(shape_id)
        .ok_or(UserError::ShapeNotFound(shape_id))?;
    let old_shape = state
        .document
        .shape_at(shape_index)
        .cloned()
        .ok_or(UserError::ShapeNotFound(shape_id))?;

    // Descending order keeps each removal index valid for the removals after it.
    anchor_indices.sort_unstable_by(|a, b| b.cmp(a));
    anchor_indices.dedup();

    let result = anchor_deletion_result(&old_shape, &anchor_indices);
    Ok(AnchorDeletion {
        shape_id,
        shape_index,
        old_shape,
        result,
    })
}

/// Decide whether removing these anchors removes the shape entirely.
fn anchor_deletion_result(old_shape: &Shape, anchor_indices: &[usize]) -> AnchorDeletionResult {
    let remaining_anchors = old_shape
        .path
        .anchors
        .len()
        .saturating_sub(anchor_indices.len());

    if remaining_anchors < 2 {
        // Shape would have fewer than 2 anchors, remove it entirely
        return AnchorDeletionResult::Removed;
    }

    // Create new shape with anchors removed
    let mut new_shape = old_shape.clone();
    for &index in anchor_indices {
        remove_anchor_from_path(&mut new_shape.path, index);
    }
    AnchorDeletionResult::Modified(new_shape)
}

/// Remove an anchor from a path at the given index, along with its corresponding segment.
fn remove_anchor_from_path(path: &mut PathGeom, idx: usize) {
    if idx < path.anchors.len() {
        path.anchors.remove(idx);
        // Also remove corresponding segment (if not the last anchor)
        if idx < path.segments.len() {
            path.segments.remove(idx);
        } else if !path.segments.is_empty() {
            path.segments.pop();
        }
    }
}

/// Replace the shape currently at the recorded document index.
/// An out-of-range index returns `InvalidOperation`; the old shape's identity
/// is not checked, so a saved index addresses its current occupant after reordering.
fn apply_shape_replacement(
    doc: &mut Document,
    replacement: &ShapeReplacement,
) -> Result<(), UserError> {
    let shapes_len = doc.len();
    let shape = doc.shape_at_mut(replacement.shape_index).ok_or_else(|| {
        UserError::InvalidOperation(format!(
            "shape replacement index {} out of range (len = {})",
            replacement.shape_index, shapes_len
        ))
    })?;
    *shape = replacement.new_shape.clone();
    Ok(())
}

/// Swap the old and new shapes to form the replacement needed for undo.
/// A prepared replacement from A to B becomes a replacement from B to A.
fn create_swapped_replacement(replacement: &ShapeReplacement) -> ShapeReplacement {
    ShapeReplacement {
        shape_index: replacement.shape_index,
        old_shape: replacement.new_shape.clone(),
        new_shape: replacement.old_shape.clone(),
    }
}

/// Apply a prepared replacement and build its inverse, propagating index errors.
/// Applying A-to-B returns an inverse that restores B-to-A.
fn apply_shape_replacement_with_inverse(
    doc: &mut Document,
    replacement: &ShapeReplacement,
    command_name: &'static str,
    create_inverse: impl FnOnce(&'static str, ShapeReplacement) -> CommandInverse,
) -> Result<CommandInverse, UserError> {
    apply_shape_replacement(doc, replacement)?;
    Ok(create_inverse(
        command_name,
        create_swapped_replacement(replacement),
    ))
}

/// Macro to generate shape replacement functions with inverses
macro_rules! shape_replacement_command {
    ($fn_name:ident, $inverse_variant:ident) => {
        pub(super) fn $fn_name(
            doc: &mut Document,
            replacement: &ShapeReplacement,
            command_name: &'static str,
        ) -> Result<CommandInverse, UserError> {
            apply_shape_replacement_with_inverse(doc, replacement, command_name, |name, repl| {
                CommandInverse::$inverse_variant {
                    command_name: name,
                    replacement: repl,
                }
            })
        }
    };
}

shape_replacement_command!(apply_insert_anchor, RemoveAnchor);
shape_replacement_command!(apply_insert_anchor_on_segment, RemoveAnchorFromSegment);

/// Restore the saved shape when undoing an anchor insertion.
/// Undoing an insertion therefore restores the path from before the split.
pub(super) fn apply_remove_anchor(
    doc: &mut Document,
    replacement: &ShapeReplacement,
) -> Result<(), UserError> {
    apply_shape_replacement(doc, replacement)
}

/// Apply anchor deletions, returning the inverse command for undo.
///
/// # Errors
///
/// Returns `UserError::InvalidOperation` if any shape index is out of range.
pub(super) fn apply_delete_anchors(
    doc: &mut Document,
    deletions: &[AnchorDeletion],
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    // Sort by index in reverse order for safe removal
    let mut sorted_deletions: Vec<_> = deletions.iter().collect();
    sorted_deletions.sort_by_key(|deletion| std::cmp::Reverse(deletion.shape_index));

    for deletion in &sorted_deletions {
        let shapes_len = doc.len();
        match &deletion.result {
            AnchorDeletionResult::Modified(new_shape) => {
                let shape = doc.shape_at_mut(deletion.shape_index).ok_or_else(|| {
                    UserError::InvalidOperation(format!(
                        "anchor deletion shape index {} out of range (len = {})",
                        deletion.shape_index, shapes_len
                    ))
                })?;
                *shape = new_shape.clone();
            }
            AnchorDeletionResult::Removed => {
                if deletion.shape_index >= shapes_len {
                    return Err(UserError::InvalidOperation(format!(
                        "anchor deletion shape removal index {} out of range (len = {})",
                        deletion.shape_index, shapes_len
                    )));
                }
                let _ = doc.remove_shape(deletion.shape_index);
            }
        }
    }

    // Create inverse restorations
    let restorations = deletions
        .iter()
        .map(|d| AnchorRestoration {
            shape_id: d.shape_id,
            shape_index: d.shape_index,
            restoration: match &d.result {
                AnchorDeletionResult::Modified(new_shape) => {
                    AnchorRestorationKind::RestoreModified {
                        from: new_shape.clone(),
                        to: d.old_shape.clone(),
                    }
                }
                AnchorDeletionResult::Removed => AnchorRestorationKind::RestoreRemoved {
                    shape: d.old_shape.clone(),
                },
            },
        })
        .collect();

    Ok(CommandInverse::RestoreAnchors {
        command_name,
        restorations,
    })
}

/// Apply anchor restorations (for undo).
///
/// # Errors
///
/// Returns `UserError::InvalidOperation` if any shape index is out of range.
pub(super) fn apply_restore_anchors(
    doc: &mut Document,
    restorations: &[AnchorRestoration],
) -> Result<(), UserError> {
    // Sort by index in forward order for safe insertion
    let mut sorted_restorations: Vec<_> = restorations.iter().collect();
    sorted_restorations.sort_by_key(|r| r.shape_index);

    for restoration in sorted_restorations {
        let shapes_len = doc.len();
        match &restoration.restoration {
            AnchorRestorationKind::RestoreModified { to, .. } => {
                let shape = doc.shape_at_mut(restoration.shape_index).ok_or_else(|| {
                    UserError::InvalidOperation(format!(
                        "anchor restoration shape index {} out of range (len = {})",
                        restoration.shape_index, shapes_len
                    ))
                })?;
                *shape = to.clone();
            }
            AnchorRestorationKind::RestoreRemoved { shape } => {
                if restoration.shape_index > shapes_len {
                    return Err(UserError::InvalidOperation(format!(
                        "anchor restoration shape insertion index {} out of range (len = {})",
                        restoration.shape_index, shapes_len
                    )));
                }
                let _shape_id = doc
                    .insert_shape(restoration.shape_index, shape.clone())
                    .ok_or_else(|| {
                        UserError::InvalidOperation(format!(
                            "anchor restoration failed to insert at index {} (len = {})",
                            restoration.shape_index, shapes_len
                        ))
                    })?;
            }
        }
    }
    Ok(())
}

shape_replacement_command!(apply_close_path, ReopenPath);

/// Apply a shape replacement recorded for reopening a closed path.
///
/// Shares the generic replacement logic so the reopen and close commands undo
/// through the same restoration path.
pub(super) fn apply_reopen_path(
    doc: &mut Document,
    replacement: &ShapeReplacement,
) -> Result<(), UserError> {
    apply_shape_replacement(doc, replacement)
}
