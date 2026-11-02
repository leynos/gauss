//! Drag-start capture for shape, anchor, and handle gestures.

use super::{
    super::{
        SelectAnchorHit,
        SelectHandleHit,
        SelectHandleHitKind,
        SelectShapeHit,
        selection::selected_shape_ids_for_drag,
    },
    AnchorDragState,
    DraggedShape,
    HandleDragState,
    SelectDragDocumentSnapshot,
    ShapesDragState,
};
use crate::model::{SelItem, Selection, Vec2};

/// Capture the selected shapes, or only the hit shape, for a body drag.
///
/// The selection moves as a group when it contains the hit shape; otherwise
/// the hit shape alone is captured. Missing or mismatched snapshot entries
/// prevent creation of a partial drag state.
pub(super) fn start_shapes_drag(
    snapshot: &SelectDragDocumentSnapshot,
    selection: &Selection,
    cursor_world: Vec2,
    hit: SelectShapeHit,
) -> Option<ShapesDragState> {
    let drag_all_selected = selection.contains(&SelItem::Shape(hit.shape_id));
    let shape_ids = if drag_all_selected {
        selected_shape_ids_for_drag(selection)
    } else {
        vec![hit.shape_id]
    };

    let mut shapes = Vec::new();
    let mut hit_index_hint = Some(hit.shape_index).filter(|index| {
        snapshot
            .shape_at(*index)
            .is_some_and(|shape| shape.id == hit.shape_id)
    });

    for shape_id in shape_ids {
        let index = if shape_id == hit.shape_id {
            hit_index_hint
                .take()
                .or_else(|| snapshot.find_index(shape_id))?
        } else {
            snapshot.find_index(shape_id)?
        };

        let shape = snapshot.shape_at(index)?;
        if shape.id != shape_id {
            continue;
        }

        shapes.push(DraggedShape {
            shape: shape_id,
            index,
            original_anchors: shape.anchors.clone(),
        });
    }

    (!shapes.is_empty()).then_some(ShapesDragState {
        start_cursor_world: cursor_world,
        shapes,
    })
}

/// Capture an anchor and its attached handles for a detail drag.
///
/// The hit's identity and indices must still match the drag-start snapshot;
/// otherwise the stale hit returns `None`.
pub(super) fn start_anchor_drag(
    snapshot: &SelectDragDocumentSnapshot,
    hit: SelectAnchorHit,
    cursor_world: Vec2,
) -> Option<AnchorDragState> {
    let shape = snapshot.shape_at(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }
    let anchor = shape.anchors.get(hit.anchor_index)?.clone();

    Some(AnchorDragState {
        shape: hit.shape_id,
        shape_index: hit.shape_index,
        anchor_index: hit.anchor_index,
        start_cursor_world: cursor_world,
        original_anchor: anchor,
    })
}

/// Capture the selected endpoint of an anchor handle for dragging.
///
/// A handle without a stored endpoint cannot begin a drag and returns `None`.
pub(super) fn start_handle_drag(
    snapshot: &SelectDragDocumentSnapshot,
    hit: SelectHandleHit,
    cursor_world: Vec2,
) -> Option<HandleDragState> {
    let shape = snapshot.shape_at(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }

    let anchor = shape.anchors.get(hit.anchor_index)?;
    let original_handle = match hit.kind {
        SelectHandleHitKind::In => anchor.handle_in?,
        SelectHandleHitKind::Out => anchor.handle_out?,
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
