//! Drag preview helpers extracted from `drag` to keep modules focused.

use crate::model::{Anchor, Document, Shape, Vec2};

use super::{
    AnchorDragState, HandleDragState, SelectDragState, SelectHandleHitKind, ShapesDragState,
};

pub(super) fn apply_drag_preview(
    doc: &mut Document,
    drag_state: &SelectDragState,
    cursor_world: Vec2,
) -> bool {
    match drag_state {
        SelectDragState::Shapes(shape_drag) => {
            apply_shapes_drag_preview(doc, shape_drag, cursor_world)
        }
        SelectDragState::Anchor(anchor_drag) => {
            apply_anchor_drag_preview(doc, anchor_drag, cursor_world)
        }
        SelectDragState::Handle(handle_drag) => {
            apply_handle_drag_preview(doc, handle_drag, cursor_world)
        }
    }
}

pub(super) fn restore_drag_preview(doc: &mut Document, drag_state: &SelectDragState) -> bool {
    match drag_state {
        SelectDragState::Shapes(shape_drag) => {
            apply_shapes_drag_to_doc(doc, shape_drag, Vec2::ZERO)
        }
        SelectDragState::Anchor(anchor_drag) => {
            apply_anchor_drag_to_doc(doc, anchor_drag, Vec2::ZERO)
        }
        SelectDragState::Handle(handle_drag) => {
            apply_handle_drag_to_doc(doc, handle_drag, Vec2::ZERO)
        }
    }
}

fn apply_shapes_drag_preview(
    doc: &mut Document,
    drag: &ShapesDragState,
    cursor_world: Vec2,
) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_shapes_drag_to_doc(doc, drag, delta)
}

fn apply_shapes_drag_to_doc(doc: &mut Document, drag: &ShapesDragState, delta: Vec2) -> bool {
    let mut did_update_any = false;

    for dragged in &drag.shapes {
        let Some(shape) = doc.shape_at_mut(dragged.index) else {
            continue;
        };
        if shape.id != dragged.shape {
            continue;
        }

        did_update_any |= restore_shape_anchors(shape, &dragged.original_anchors, delta);
    }

    did_update_any
}

fn apply_anchor_drag_preview(
    doc: &mut Document,
    drag: &AnchorDragState,
    cursor_world: Vec2,
) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_anchor_drag_to_doc(doc, drag, delta)
}

fn apply_anchor_drag_to_doc(doc: &mut Document, drag: &AnchorDragState, delta: Vec2) -> bool {
    let Some(shape) = doc.shape_at_mut(drag.shape_index) else {
        return false;
    };
    if shape.id != drag.shape {
        return false;
    }
    let Some(anchor) = shape.path.anchors.get_mut(drag.anchor_index) else {
        return false;
    };

    anchor.pos = drag.original_anchor.pos.add(delta);
    anchor.handle_in = drag.original_anchor.handle_in.map(|p| p.add(delta));
    anchor.handle_out = drag.original_anchor.handle_out.map(|p| p.add(delta));
    true
}

fn apply_handle_drag_preview(
    doc: &mut Document,
    drag: &HandleDragState,
    cursor_world: Vec2,
) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_handle_drag_to_doc(doc, drag, delta)
}

fn apply_handle_drag_to_doc(doc: &mut Document, drag: &HandleDragState, delta: Vec2) -> bool {
    let Some(shape) = doc.shape_at_mut(drag.shape_index) else {
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
        SelectHandleHitKind::In => {
            anchor.handle_in = Some(moved);
        }
        SelectHandleHitKind::Out => {
            anchor.handle_out = Some(moved);
        }
    }

    true
}

fn restore_shape_anchors(shape: &mut Shape, original: &[Anchor], delta: Vec2) -> bool {
    if shape.path.anchors.len() != original.len() {
        return false;
    }

    for (current, start) in shape.path.anchors.iter_mut().zip(original.iter()) {
        current.pos = start.pos.add(delta);
        current.handle_in = start.handle_in.map(|p| p.add(delta));
        current.handle_out = start.handle_out.map(|p| p.add(delta));
    }

    true
}
