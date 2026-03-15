//! Shape, anchor, and handle movement command helpers.

use crate::model::{Anchor, Document, Vec2};

use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::{AnchorMovement, HandleKind, HandleMovement, ShapeMovement};

/// Translate an anchor position and its handles by the given delta.
#[expect(clippy::missing_const_for_fn, reason = "uses mutable reference")]
fn translate_anchor(anchor: &mut Anchor, delta: Vec2) {
    anchor.pos = anchor.pos.add(delta);
    if let Some(h) = anchor.handle_in.as_mut() {
        *h = h.add(delta);
    }
    if let Some(h) = anchor.handle_out.as_mut() {
        *h = h.add(delta);
    }
}

#[derive(Clone, Copy)]
struct AnchorTarget {
    shape_id: crate::model::ShapeId,
    anchor_index: usize,
}

/// Apply a mutation to an anchor, returning an error if missing.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if the shape does not exist.
/// Returns `UserError::AnchorNotFound` if the anchor index is out of range.
fn with_anchor_mut<F>(doc: &mut Document, target: AnchorTarget, mutate: F) -> Result<(), UserError>
where
    F: FnOnce(&mut Anchor),
{
    let AnchorTarget {
        shape_id,
        anchor_index,
    } = target;
    let shape = doc
        .shape_mut(shape_id)
        .ok_or(UserError::ShapeNotFound(shape_id))?;
    let anchor = shape
        .path
        .anchors
        .get_mut(anchor_index)
        .ok_or(UserError::AnchorNotFound(shape_id, anchor_index))?;
    mutate(anchor);
    Ok(())
}

/// Validate that all referenced shapes exist in the document.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
fn validate_shapes_exist(doc: &Document, movements: &[ShapeMovement]) -> Result<(), UserError> {
    movements.iter().try_for_each(|movement| {
        doc.find_index(movement.shape_id)
            .ok_or(UserError::ShapeNotFound(movement.shape_id))
            .map(|_| ())
    })
}

/// Apply shape movements, returning the inverse command for undo.
///
/// Pre-validates all shape IDs before applying any changes to ensure atomicity:
/// either all changes succeed or none are applied.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
pub(super) fn apply_move_shapes(
    doc: &mut Document,
    movements: &[ShapeMovement],
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    // Pre-validate: ensure all shapes exist before applying any changes.
    validate_shapes_exist(doc, movements)?;

    // Apply changes (safe now that all shapes are validated).
    for movement in movements {
        let shape = doc
            .shape_mut(movement.shape_id)
            .ok_or(UserError::ShapeNotFound(movement.shape_id))?;
        for anchor in &mut shape.path.anchors {
            translate_anchor(anchor, movement.delta);
        }
    }

    // Create inverse with negated deltas
    let inverse_movements = movements
        .iter()
        .map(|m| ShapeMovement {
            shape_id: m.shape_id,
            delta: Vec2::new(-m.delta.x, -m.delta.y),
        })
        .collect();

    Ok(CommandInverse::MoveShapesBack {
        command_name,
        movements: inverse_movements,
    })
}

/// Apply the inverse of shape movements (for undo).
///
/// Pre-validates all shape IDs before applying any changes to ensure atomicity:
/// either all changes succeed or none are applied.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if any referenced shape does not exist.
pub(super) fn apply_move_shapes_back(
    doc: &mut Document,
    movements: &[ShapeMovement],
) -> Result<(), UserError> {
    // Pre-validate: ensure all shapes exist before applying any changes.
    validate_shapes_exist(doc, movements)?;

    // Apply changes (safe now that all shapes are validated).
    for movement in movements {
        let shape = doc
            .shape_mut(movement.shape_id)
            .ok_or(UserError::ShapeNotFound(movement.shape_id))?;
        for anchor in &mut shape.path.anchors {
            translate_anchor(anchor, movement.delta);
        }
    }
    Ok(())
}

/// Apply an anchor movement, returning the inverse command for undo.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if the shape does not exist.
/// Returns `UserError::AnchorNotFound` if the anchor index is out of range.
pub(super) fn apply_move_anchor(
    doc: &mut Document,
    movement: &AnchorMovement,
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        |anchor| translate_anchor(anchor, movement.delta),
    )?;

    Ok(CommandInverse::MoveAnchorBack {
        command_name,
        movement: AnchorMovement {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
            original: movement.original.clone(),
            delta: Vec2::new(-movement.delta.x, -movement.delta.y),
        },
    })
}

/// Apply the inverse of an anchor movement (for undo).
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if the shape does not exist.
/// Returns `UserError::AnchorNotFound` if the anchor index is out of range.
pub(super) fn apply_move_anchor_back(
    doc: &mut Document,
    movement: &AnchorMovement,
) -> Result<(), UserError> {
    // Restore the original anchor state
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        |anchor| *anchor = movement.original.clone(),
    )
}

/// Apply a handle movement, returning the inverse command for undo.
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if the shape does not exist.
/// Returns `UserError::AnchorNotFound` if the anchor index is out of range.
pub(super) fn apply_move_handle(
    doc: &mut Document,
    movement: &HandleMovement,
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        |anchor| match movement.kind {
            HandleKind::In => anchor.handle_in = movement.to,
            HandleKind::Out => anchor.handle_out = movement.to,
        },
    )?;

    Ok(CommandInverse::MoveHandleBack {
        command_name,
        movement: HandleMovement {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
            kind: movement.kind,
            from: movement.to,
            to: movement.from,
        },
    })
}

/// Apply the inverse of a handle movement (for undo).
///
/// # Errors
///
/// Returns `UserError::ShapeNotFound` if the shape does not exist.
/// Returns `UserError::AnchorNotFound` if the anchor index is out of range.
pub(super) fn apply_move_handle_back(
    doc: &mut Document,
    movement: &HandleMovement,
) -> Result<(), UserError> {
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        |anchor| match movement.kind {
            HandleKind::In => anchor.handle_in = movement.to,
            HandleKind::Out => anchor.handle_out = movement.to,
        },
    )
}
