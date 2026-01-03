//! Handle dragging behaviour for Phase 0 manipulate mode.
//!
//! Handle drags are expressed as Commands so they can be undone via command
//! history.

use crate::model::{Document, HandleKind, HandleMovement, ShapeId, Vec2};

use super::hit_test::{HandleHit, HandleHitKind};

#[derive(Clone, Debug)]
pub(super) struct HandleDragState {
    pub(super) shape: ShapeId,
    pub(super) shape_index: usize,
    pub(super) anchor_index: usize,
    pub(super) kind: HandleHitKind,
    pub(super) start_cursor_world: Vec2,
    pub(super) original_handle: Vec2,
}

pub(super) fn start_handle_drag(
    doc: &Document,
    hit: HandleHit,
    cursor_world: Vec2,
) -> Option<HandleDragState> {
    let shape = doc.shapes.get(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }

    let anchor = shape.path.anchors.get(hit.anchor_index)?;
    let original_handle = match hit.kind {
        HandleHitKind::In => anchor.handle_in?,
        HandleHitKind::Out => anchor.handle_out?,
    };

    Some(HandleDragState {
        shape: hit.shape_id,
        shape_index: hit.shape_index,
        anchor_index: hit.anchor_index,
        kind: hit.kind,
        start_cursor_world: cursor_world,
        original_handle,
    })
}

pub(super) fn apply_handle_drag_preview(
    doc: &mut Document,
    drag: &HandleDragState,
    cursor_world: Vec2,
) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_handle_drag_to_doc(doc, drag, delta)
}

pub(super) fn finish_handle_drag(
    doc: &mut Document,
    drag: &HandleDragState,
    cursor_world: Vec2,
) -> Option<HandleMovement> {
    let delta = cursor_world.sub(drag.start_cursor_world);

    let did_move = delta.x.abs() > f32::EPSILON || delta.y.abs() > f32::EPSILON;
    let did_restore = apply_handle_drag_to_doc(doc, drag, Vec2::ZERO);
    if !did_restore || !did_move {
        return None;
    }

    let to = drag.original_handle.add(delta);
    Some(HandleMovement {
        shape_id: drag.shape,
        anchor_index: drag.anchor_index,
        kind: match drag.kind {
            HandleHitKind::In => HandleKind::In,
            HandleHitKind::Out => HandleKind::Out,
        },
        from: Some(drag.original_handle),
        to: Some(to),
    })
}

fn apply_handle_drag_to_doc(doc: &mut Document, drag: &HandleDragState, delta: Vec2) -> bool {
    let Some(shape) = doc.shapes.get_mut(drag.shape_index) else {
        return false;
    };
    if shape.id != drag.shape {
        return false;
    }

    let Some(anchor) = shape.path.anchors.get_mut(drag.anchor_index) else {
        return false;
    };

    let moved = drag.original_handle.add(delta);
    match drag.kind {
        HandleHitKind::In => {
            anchor.handle_in = Some(moved);
        }
        HandleHitKind::Out => {
            anchor.handle_out = Some(moved);
        }
    }

    true
}
