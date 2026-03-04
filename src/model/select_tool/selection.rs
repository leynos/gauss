//! Selection decision helpers for `SelectTool` transitions.
//!
//! This module is model-layer and deterministic so selection transitions can be
//! validated via unit and behaviour tests without GPUI dependencies.

use crate::model::{SelItem, Selection, ShapeId};

use super::{SelectHandleHit, SelectHandleHitKind, SelectPointerHit, SelectShapeHit};

pub(super) fn selection_for_hit(
    previous: &Selection,
    hit: SelectPointerHit,
    is_shift_held: bool,
) -> Selection {
    if is_shift_held {
        toggle_selection_for_hit(previous, hit)
    } else {
        selection_for_non_shift_hit(previous, hit)
    }
}

pub(super) fn can_drag_shape_bbox(previous: &Selection, hit: SelectPointerHit) -> bool {
    match hit {
        SelectPointerHit::Shape(SelectShapeHit { shape_id, .. }) => {
            previous.contains(&SelItem::Shape(shape_id))
        }
        _ => true,
    }
}

pub(super) fn selected_shape_ids_for_drag(selection: &Selection) -> Vec<ShapeId> {
    selection
        .items
        .iter()
        .filter_map(|item| match item {
            SelItem::Shape(shape_id) => Some(*shape_id),
            _ => None,
        })
        .collect()
}

fn toggle_selection_for_hit(current: &Selection, hit: SelectPointerHit) -> Selection {
    let Some(item) = toggle_item_for_hit(hit) else {
        return current.clone();
    };

    toggle_item_with_parent(current, item)
}

fn toggle_item_with_parent(current: &Selection, item: SelItem) -> Selection {
    match item {
        SelItem::Shape(shape_id) => toggle_shape_with_children(current, shape_id),
        _ => toggle_detail_item(current, item),
    }
}

fn toggle_shape_with_children(current: &Selection, shape_id: ShapeId) -> Selection {
    if current.contains(&SelItem::Shape(shape_id)) {
        Selection {
            items: current
                .items
                .iter()
                .filter(|item| shape_id_of_item(item) != shape_id)
                .cloned()
                .collect(),
        }
    } else {
        let mut selection = current.clone();
        selection.items.push(SelItem::Shape(shape_id));
        selection
    }
}

fn toggle_detail_item(current: &Selection, item: SelItem) -> Selection {
    let mut selection = current.clone();

    let shape_id = shape_id_of_item(&item);
    if !selection.contains(&SelItem::Shape(shape_id)) {
        selection.items.push(SelItem::Shape(shape_id));
    }

    if let Some(pos) = selection
        .items
        .iter()
        .position(|existing| existing == &item)
    {
        selection.items.remove(pos);
    } else {
        selection.items.push(item);
    }

    selection
}

fn selection_for_non_shift_hit(previous_selection: &Selection, hit: SelectPointerHit) -> Selection {
    match hit {
        SelectPointerHit::None => Selection::empty(),
        SelectPointerHit::Shape(SelectShapeHit { shape_id, .. }) => {
            if previous_selection.contains(&SelItem::Shape(shape_id)) {
                previous_selection.clone()
            } else {
                Selection {
                    items: vec![SelItem::Shape(shape_id)],
                }
            }
        }
        SelectPointerHit::Handle(handle_hit) => selection_for_shape_detail_hit(
            previous_selection,
            handle_hit.shape_id,
            sel_item_for_handle_hit(handle_hit),
        ),
        SelectPointerHit::Anchor(anchor_hit) => selection_for_shape_detail_hit(
            previous_selection,
            anchor_hit.shape_id,
            SelItem::Anchor {
                shape: anchor_hit.shape_id,
                anchor: anchor_hit.anchor_index,
            },
        ),
        SelectPointerHit::Segment(segment_hit) => selection_for_shape_detail_hit(
            previous_selection,
            segment_hit.shape_id,
            SelItem::Segment {
                shape: segment_hit.shape_id,
                seg: segment_hit.seg_index,
            },
        ),
    }
}

fn selection_for_shape_detail_hit(
    previous_selection: &Selection,
    shape_id: ShapeId,
    detail: SelItem,
) -> Selection {
    let mut items = Vec::new();
    let mut hit_shape_previously_selected = false;
    for item in &previous_selection.items {
        let SelItem::Shape(id) = item else {
            continue;
        };

        if *id == shape_id {
            hit_shape_previously_selected = true;
        }
        items.push(SelItem::Shape(*id));
    }

    if !hit_shape_previously_selected {
        items.clear();
        items.push(SelItem::Shape(shape_id));
    }

    items.push(detail);
    Selection { items }
}

const fn toggle_item_for_hit(hit: SelectPointerHit) -> Option<SelItem> {
    match hit {
        SelectPointerHit::Handle(handle_hit) => Some(sel_item_for_handle_hit(handle_hit)),
        SelectPointerHit::Anchor(anchor_hit) => Some(SelItem::Anchor {
            shape: anchor_hit.shape_id,
            anchor: anchor_hit.anchor_index,
        }),
        SelectPointerHit::Segment(segment_hit) => Some(SelItem::Segment {
            shape: segment_hit.shape_id,
            seg: segment_hit.seg_index,
        }),
        SelectPointerHit::Shape(SelectShapeHit { shape_id, .. }) => Some(SelItem::Shape(shape_id)),
        SelectPointerHit::None => None,
    }
}

const fn sel_item_for_handle_hit(hit: SelectHandleHit) -> SelItem {
    match hit.kind {
        SelectHandleHitKind::In => SelItem::HandleIn {
            shape: hit.shape_id,
            anchor: hit.anchor_index,
        },
        SelectHandleHitKind::Out => SelItem::HandleOut {
            shape: hit.shape_id,
            anchor: hit.anchor_index,
        },
    }
}

const fn shape_id_of_item(item: &SelItem) -> ShapeId {
    match item {
        SelItem::Shape(id) => *id,
        SelItem::Anchor { shape, .. }
        | SelItem::HandleIn { shape, .. }
        | SelItem::HandleOut { shape, .. }
        | SelItem::Segment { shape, .. } => *shape,
    }
}
