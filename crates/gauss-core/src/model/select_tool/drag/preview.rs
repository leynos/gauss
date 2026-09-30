//! Drag preview helpers extracted from `drag` to keep modules focused.

use crate::model::{Anchor, Document, Shape, Vec2};

use super::{
    AnchorDragState, HandleDragState, SelectDragState, SelectHandleHitKind, ShapesDragState,
};

/// Move the document geometry to the position implied by the current cursor.
///
/// Returns `true` when a snapshot-matched target was updated; missing or stale
/// indices leave that target untouched.
/// For a matching shape, moving the cursor applies its start-relative delta.
pub(super) fn apply_drag_preview(
    doc: &mut Document,
    drag_state: &SelectDragState,
    cursor_world: Vec2,
) -> bool {
    apply_drag_internal(doc, drag_state, Some(cursor_world))
}

/// Return previewed geometry to the drag-start snapshot.
///
/// For example, after a cursor preview moves an anchor, this restores the
/// original anchor and its attached handles before the final command applies.
pub(super) fn restore_drag_preview(doc: &mut Document, drag_state: &SelectDragState) -> bool {
    apply_drag_internal(doc, drag_state, None)
}

/// Dispatch one preview or restore update to the geometry for the active drag.
///
/// `Some(cursor)` applies cursor displacement; `None` uses zero displacement
/// to restore the captured starting geometry.
fn apply_drag_internal(
    doc: &mut Document,
    drag_state: &SelectDragState,
    cursor_world: Option<Vec2>,
) -> bool {
    match drag_state {
        SelectDragState::Shapes(shape_drag) => apply_shapes_drag_to_doc(
            doc,
            shape_drag,
            cursor_world.map_or(Vec2::ZERO, |cursor| {
                cursor.sub(shape_drag.start_cursor_world)
            }),
        ),
        SelectDragState::Anchor(anchor_drag) => apply_anchor_drag_to_doc(
            doc,
            anchor_drag,
            cursor_world.map_or(Vec2::ZERO, |cursor| {
                cursor.sub(anchor_drag.start_cursor_world)
            }),
        ),
        SelectDragState::Handle(handle_drag) => apply_handle_drag_to_doc(
            doc,
            handle_drag,
            cursor_world.map_or(Vec2::ZERO, |cursor| {
                cursor.sub(handle_drag.start_cursor_world)
            }),
        ),
    }
}

/// Update each still-matching shape in a group drag from its original anchors.
/// Each captured shape receives the same cursor delta.
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

/// Move one anchor and translate both attached handles by the same delta.
/// A `(2, 0)` delta shifts the anchor and both endpoints two units right.
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

/// Move only the captured incoming or outgoing handle endpoint.
/// An incoming-handle drag leaves the anchor's outgoing handle unchanged.
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

/// Rebuild a shape's anchor geometry from its snapshot plus the preview delta.
///
/// The anchor counts must agree; otherwise the shape is left unchanged and the
/// caller can report that no update occurred.
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
