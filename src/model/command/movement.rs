//! Shape, anchor, and handle movement command helpers.

use crate::model::{Anchor, Document, Vec2};

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

pub(super) fn apply_move_shapes(
    doc: &mut Document,
    movements: &[ShapeMovement],
    command_name: &'static str,
) -> CommandInverse {
    for movement in movements {
        if let Some(shape) = doc.get_mut(movement.shape_id) {
            for anchor in &mut shape.path.anchors {
                translate_anchor(anchor, movement.delta);
            }
        } else {
            debug_assert!(false, "missing shape for move: {:?}", movement.shape_id);
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

    CommandInverse::MoveShapesBack {
        command_name,
        movements: inverse_movements,
    }
}

pub(super) fn apply_move_shapes_back(doc: &mut Document, movements: &[ShapeMovement]) {
    // Same logic as apply_move_shapes, deltas are already negated
    for movement in movements {
        if let Some(shape) = doc.get_mut(movement.shape_id) {
            for anchor in &mut shape.path.anchors {
                translate_anchor(anchor, movement.delta);
            }
        } else {
            debug_assert!(
                false,
                "missing shape for move back: {:?}",
                movement.shape_id
            );
        }
    }
}

pub(super) fn apply_move_anchor(
    doc: &mut Document,
    movement: &AnchorMovement,
    command_name: &'static str,
) -> CommandInverse {
    if let Some(shape) = doc.get_mut(movement.shape_id)
        && let Some(anchor) = shape.path.anchors.get_mut(movement.anchor_index)
    {
        translate_anchor(anchor, movement.delta);
    } else {
        debug_assert!(
            false,
            "missing shape or anchor for move anchor: {:?} / {}",
            movement.shape_id, movement.anchor_index
        );
    }

    CommandInverse::MoveAnchorBack {
        command_name,
        movement: AnchorMovement {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
            original: movement.original.clone(),
            delta: Vec2::new(-movement.delta.x, -movement.delta.y),
        },
    }
}

pub(super) fn apply_move_anchor_back(doc: &mut Document, movement: &AnchorMovement) {
    // Restore the original anchor state
    if let Some(shape) = doc.get_mut(movement.shape_id)
        && let Some(anchor) = shape.path.anchors.get_mut(movement.anchor_index)
    {
        *anchor = movement.original.clone();
    } else {
        debug_assert!(
            false,
            "missing shape or anchor for move anchor back: {:?} / {}",
            movement.shape_id, movement.anchor_index
        );
    }
}

pub(super) fn apply_move_handle(
    doc: &mut Document,
    movement: &HandleMovement,
    command_name: &'static str,
) -> CommandInverse {
    if let Some(shape) = doc.get_mut(movement.shape_id)
        && let Some(anchor) = shape.path.anchors.get_mut(movement.anchor_index)
    {
        match movement.kind {
            HandleKind::In => anchor.handle_in = movement.to,
            HandleKind::Out => anchor.handle_out = movement.to,
        }
    } else {
        debug_assert!(
            false,
            "missing shape or anchor for move handle: {:?} / {}",
            movement.shape_id, movement.anchor_index
        );
    }

    CommandInverse::MoveHandleBack {
        command_name,
        movement: HandleMovement {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
            kind: movement.kind,
            from: movement.to,
            to: movement.from,
        },
    }
}

pub(super) fn apply_move_handle_back(doc: &mut Document, movement: &HandleMovement) {
    if let Some(shape) = doc.get_mut(movement.shape_id)
        && let Some(anchor) = shape.path.anchors.get_mut(movement.anchor_index)
    {
        match movement.kind {
            HandleKind::In => anchor.handle_in = movement.to,
            HandleKind::Out => anchor.handle_out = movement.to,
        }
    } else {
        debug_assert!(
            false,
            "missing shape or anchor for move handle back: {:?} / {}",
            movement.shape_id, movement.anchor_index
        );
    }
}
