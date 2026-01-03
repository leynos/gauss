//! Anchor and path command helpers.

use std::collections::HashMap;

use crate::model::{
    Anchor, Document, EngineState, PathGeom, SegmentKind, SelItem, Shape, ShapeId, Vec2,
};

use super::Command;
use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::{
    AnchorDeletion, AnchorDeletionResult, AnchorRestoration, AnchorRestorationKind,
    ShapeReplacement,
};

pub(super) fn prepare_insert_anchor_on_segment(state: &EngineState) -> Result<Command, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    // Find selected segment
    let seg_selection = state
        .selection
        .items
        .iter()
        .find(|item| matches!(item, SelItem::Segment { .. }));

    let Some(SelItem::Segment { shape, seg }) = seg_selection else {
        return Err(UserError::InvalidOperation("No segment selected"));
    };

    let Some(shape_index) = state.document.find_index(*shape) else {
        return Err(UserError::ShapeNotFound(*shape));
    };

    let Some(old_shape) = state.document.shapes.get(shape_index).cloned() else {
        return Err(UserError::ShapeNotFound(*shape));
    };

    // Create new shape with inserted anchor
    let Some(new_shape) = insert_anchor_into_shape(&old_shape, *seg) else {
        return Err(UserError::InvalidOperation(
            "Cannot insert anchor on segment",
        ));
    };

    Ok(Command::InsertAnchor {
        replacement: ShapeReplacement {
            shape_index,
            old_shape,
            new_shape,
        },
    })
}

/// Insert an anchor at the midpoint of a segment.
fn insert_anchor_into_shape(shape: &Shape, seg_index: usize) -> Option<Shape> {
    let path = &shape.path;
    let seg_kind = *path.segments.get(seg_index)?;
    let start_anchor = path.anchors.get(seg_index)?;
    let end_anchor = path.anchors.get(seg_index + 1)?;

    let mut new_shape = shape.clone();

    match seg_kind {
        SegmentKind::Line => {
            // For line segments, insert midpoint with no handles
            let mid = Vec2::new(
                f32::midpoint(start_anchor.pos.x, end_anchor.pos.x),
                f32::midpoint(start_anchor.pos.y, end_anchor.pos.y),
            );
            let new_anchor = Anchor {
                pos: mid,
                handle_in: None,
                handle_out: None,
            };
            new_shape.path.anchors.insert(seg_index + 1, new_anchor);
            new_shape
                .path
                .segments
                .insert(seg_index + 1, SegmentKind::Line);
        }
        SegmentKind::Cubic => {
            // For cubic segments, use de Casteljau split at t=0.5
            let p0 = start_anchor.pos;
            let p1 = start_anchor.handle_out.unwrap_or(p0);
            let p2 = end_anchor.handle_in.unwrap_or(end_anchor.pos);
            let p3 = end_anchor.pos;

            // De Casteljau at t=0.5
            let q0 = midpoint(p0, p1);
            let q1 = midpoint(p1, p2);
            let q2 = midpoint(p2, p3);
            let r0 = midpoint(q0, q1);
            let r1 = midpoint(q1, q2);
            let s = midpoint(r0, r1);

            // Update start anchor's handle_out
            new_shape.path.anchors.get_mut(seg_index)?.handle_out = Some(q0);

            // Insert new anchor at split point
            let new_anchor = Anchor {
                pos: s,
                handle_in: Some(r0),
                handle_out: Some(r1),
            };
            new_shape.path.anchors.insert(seg_index + 1, new_anchor);

            // Update end anchor's handle_in (now at seg_index + 2 after insert)
            new_shape.path.anchors.get_mut(seg_index + 2)?.handle_in = Some(q2);

            // Insert new segment
            new_shape
                .path
                .segments
                .insert(seg_index + 1, SegmentKind::Cubic);
        }
    }

    Some(new_shape)
}

const fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(f32::midpoint(a.x, b.x), f32::midpoint(a.y, b.y))
}

pub(super) fn prepare_delete_selected_anchors(state: &EngineState) -> Result<Command, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    // Group selected anchors by shape
    let mut anchors_by_shape: HashMap<ShapeId, Vec<usize>> = HashMap::new();
    for item in &state.selection.items {
        if let SelItem::Anchor { shape, anchor } = item {
            anchors_by_shape.entry(*shape).or_default().push(*anchor);
        }
    }

    if anchors_by_shape.is_empty() {
        return Err(UserError::InvalidOperation("No anchors selected"));
    }

    let mut deletions = Vec::new();

    for (shape_id, mut anchor_indices) in anchors_by_shape {
        let Some(shape_index) = state.document.find_index(shape_id) else {
            return Err(UserError::ShapeNotFound(shape_id));
        };
        let Some(old_shape) = state.document.shapes.get(shape_index).cloned() else {
            return Err(UserError::ShapeNotFound(shape_id));
        };

        // Sort anchor indices in descending order for safe removal
        anchor_indices.sort_unstable_by(|a, b| b.cmp(a));
        anchor_indices.dedup();

        let remaining_anchors = old_shape
            .path
            .anchors
            .len()
            .saturating_sub(anchor_indices.len());

        let result = if remaining_anchors < 2 {
            // Shape would have fewer than 2 anchors, remove it entirely
            AnchorDeletionResult::Removed
        } else {
            // Create new shape with anchors removed
            let mut new_shape = old_shape.clone();
            for &idx in &anchor_indices {
                remove_anchor_from_path(&mut new_shape.path, idx);
            }
            AnchorDeletionResult::Modified(new_shape)
        };

        deletions.push(AnchorDeletion {
            shape_id,
            shape_index,
            old_shape,
            result,
        });
    }

    Ok(Command::DeleteAnchors { deletions })
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

fn apply_shape_replacement(doc: &mut Document, replacement: &ShapeReplacement) {
    if let Some(shape) = doc.shapes.get_mut(replacement.shape_index) {
        *shape = replacement.new_shape.clone();
    }
}

fn create_swapped_replacement(replacement: &ShapeReplacement) -> ShapeReplacement {
    ShapeReplacement {
        shape_index: replacement.shape_index,
        old_shape: replacement.new_shape.clone(),
        new_shape: replacement.old_shape.clone(),
    }
}

fn apply_shape_replacement_with_inverse(
    doc: &mut Document,
    replacement: &ShapeReplacement,
    command_name: &'static str,
    create_inverse: impl FnOnce(&'static str, ShapeReplacement) -> CommandInverse,
) -> CommandInverse {
    apply_shape_replacement(doc, replacement);
    create_inverse(command_name, create_swapped_replacement(replacement))
}

pub(super) fn apply_insert_anchor(
    doc: &mut Document,
    replacement: &ShapeReplacement,
    command_name: &'static str,
) -> CommandInverse {
    apply_shape_replacement_with_inverse(doc, replacement, command_name, |name, repl| {
        CommandInverse::RemoveAnchor {
            command_name: name,
            replacement: repl,
        }
    })
}

pub(super) fn apply_remove_anchor(doc: &mut Document, replacement: &ShapeReplacement) {
    apply_shape_replacement(doc, replacement);
}

pub(super) fn apply_delete_anchors(
    doc: &mut Document,
    deletions: &[AnchorDeletion],
    command_name: &'static str,
) -> CommandInverse {
    // Sort by index in reverse order for safe removal
    let mut sorted_deletions: Vec<_> = deletions.iter().collect();
    sorted_deletions.sort_by(|a, b| b.shape_index.cmp(&a.shape_index));

    for deletion in &sorted_deletions {
        match &deletion.result {
            AnchorDeletionResult::Modified(new_shape) => {
                if let Some(shape) = doc.shapes.get_mut(deletion.shape_index) {
                    *shape = new_shape.clone();
                }
            }
            AnchorDeletionResult::Removed => {
                if deletion.shape_index < doc.shapes.len() {
                    doc.shapes.remove(deletion.shape_index);
                }
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

    CommandInverse::RestoreAnchors {
        command_name,
        restorations,
    }
}

pub(super) fn apply_restore_anchors(doc: &mut Document, restorations: &[AnchorRestoration]) {
    // Sort by index in forward order for safe insertion
    let mut sorted_restorations: Vec<_> = restorations.iter().collect();
    sorted_restorations.sort_by_key(|r| r.shape_index);

    for restoration in sorted_restorations {
        match &restoration.restoration {
            AnchorRestorationKind::RestoreModified { to, .. } => {
                if let Some(shape) = doc.shapes.get_mut(restoration.shape_index) {
                    *shape = to.clone();
                }
            }
            AnchorRestorationKind::RestoreRemoved { shape } => {
                if restoration.shape_index <= doc.shapes.len() {
                    doc.shapes.insert(restoration.shape_index, shape.clone());
                }
            }
        }
    }
}

pub(super) fn apply_close_path(
    doc: &mut Document,
    replacement: &ShapeReplacement,
    command_name: &'static str,
) -> CommandInverse {
    apply_shape_replacement_with_inverse(doc, replacement, command_name, |name, repl| {
        CommandInverse::ReopenPath {
            command_name: name,
            replacement: repl,
        }
    })
}

pub(super) fn apply_reopen_path(doc: &mut Document, replacement: &ShapeReplacement) {
    apply_shape_replacement(doc, replacement);
}
