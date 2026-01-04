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

#[derive(Clone, Copy)]
struct AnchorTarget {
    shape_id: crate::model::ShapeId,
    anchor_index: usize,
}

/// Helper: Apply a mutation to an anchor if it exists, with debug assertion fallback.
fn with_anchor_mut<F>(doc: &mut Document, target: AnchorTarget, operation: &str, mutate: F)
where
    F: FnOnce(&mut Anchor),
{
    let AnchorTarget {
        shape_id,
        anchor_index,
    } = target;
    if let Some(shape) = doc.get_mut(shape_id)
        && let Some(anchor) = shape.path.anchors.get_mut(anchor_index)
    {
        mutate(anchor);
    } else {
        debug_assert!(
            false,
            "missing shape or anchor for {operation}: {shape_id:?} / {anchor_index}"
        );
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
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        "move anchor",
        |anchor| translate_anchor(anchor, movement.delta),
    );

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
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        "move anchor back",
        |anchor| *anchor = movement.original.clone(),
    );
}

pub(super) fn apply_move_handle(
    doc: &mut Document,
    movement: &HandleMovement,
    command_name: &'static str,
) -> CommandInverse {
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        "move handle",
        |anchor| match movement.kind {
            HandleKind::In => anchor.handle_in = movement.to,
            HandleKind::Out => anchor.handle_out = movement.to,
        },
    );

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
    with_anchor_mut(
        doc,
        AnchorTarget {
            shape_id: movement.shape_id,
            anchor_index: movement.anchor_index,
        },
        "move handle back",
        |anchor| match movement.kind {
            HandleKind::In => anchor.handle_in = movement.to,
            HandleKind::Out => anchor.handle_out = movement.to,
        },
    );
}
