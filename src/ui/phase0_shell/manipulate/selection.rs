//! Selection behaviour for Phase 0 manipulate mode.
//!
//! The logic here keeps selection changes isolated from drag setup so the
//! event handlers remain easy to follow.

use crate::model::{SelItem, Selection, ShapeId};
use crate::ui::selection_utils::selected_shape_ids;

use super::hit_test::{HandleHit, HandleHitKind, MouseDownHit, ShapeHit};

pub(super) fn selection_for_hit(
    previous: &Selection,
    hit: MouseDownHit,
    is_shift_held: bool,
) -> Selection {
    if is_shift_held {
        toggle_selection_for_hit(previous, hit)
    } else {
        selection_for_non_shift_hit(previous, hit)
    }
}

pub(super) fn can_drag_shape_bbox(previous: &Selection, hit: MouseDownHit) -> bool {
    match hit {
        MouseDownHit::Shape(ShapeHit { shape_id, .. }) => {
            previous.contains(&SelItem::Shape(shape_id))
        }
        _ => true,
    }
}

pub(super) fn selected_shape_ids_for_drag(selection: &Selection) -> Vec<ShapeId> {
    selected_shape_ids(selection).collect()
}

fn toggle_selection_for_hit(current: &Selection, hit: MouseDownHit) -> Selection {
    let Some(item) = toggle_item_for_hit(hit) else {
        return current.clone();
    };

    toggle_item_with_parent(current, item)
}

fn toggle_item_in_selection(mut selection: Selection, item: SelItem) -> Selection {
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
        toggle_item_in_selection(current.clone(), SelItem::Shape(shape_id))
    }
}

fn toggle_detail_item(current: &Selection, item: SelItem) -> Selection {
    let mut selection = current.clone();

    let shape_id = shape_id_of_item(&item);
    if !selection.contains(&SelItem::Shape(shape_id)) {
        selection.items.push(SelItem::Shape(shape_id));
    }

    toggle_item_in_selection(selection, item)
}

fn selection_for_non_shift_hit(previous_selection: &Selection, hit: MouseDownHit) -> Selection {
    match hit {
        MouseDownHit::None => Selection::empty(),
        MouseDownHit::Shape(ShapeHit { shape_id, .. }) => {
            // Clicking an already-selected shape should not collapse the current
            // selection. This is critical for multi-select move gestures.
            if previous_selection.contains(&SelItem::Shape(shape_id)) {
                previous_selection.clone()
            } else {
                Selection {
                    items: vec![SelItem::Shape(shape_id)],
                }
            }
        }
        MouseDownHit::Handle(handle_hit) => selection_for_shape_detail_hit(
            previous_selection,
            handle_hit.shape_id,
            sel_item_for_handle_hit(handle_hit),
        ),
        MouseDownHit::Anchor(anchor_hit) => selection_for_shape_detail_hit(
            previous_selection,
            anchor_hit.shape_id,
            SelItem::Anchor {
                shape: anchor_hit.shape_id,
                anchor: anchor_hit.anchor_index,
            },
        ),
        MouseDownHit::Segment(segment_hit) => selection_for_shape_detail_hit(
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
    let previous_shapes = shapes_only(previous_selection);

    let mut items = if previous_shapes.contains(&SelItem::Shape(shape_id)) {
        // Preserve existing multi-select shape set when editing within one of
        // the selected shapes.
        previous_shapes.items
    } else {
        vec![SelItem::Shape(shape_id)]
    };

    items.push(detail);
    Selection { items }
}

fn shapes_only(selection: &Selection) -> Selection {
    let mut items = Vec::new();
    for item in &selection.items {
        let SelItem::Shape(id) = item else {
            continue;
        };
        items.push(SelItem::Shape(*id));
    }
    Selection { items }
}

const fn toggle_item_for_hit(hit: MouseDownHit) -> Option<SelItem> {
    match hit {
        MouseDownHit::Handle(handle_hit) => Some(sel_item_for_handle_hit(handle_hit)),
        MouseDownHit::Anchor(anchor_hit) => Some(SelItem::Anchor {
            shape: anchor_hit.shape_id,
            anchor: anchor_hit.anchor_index,
        }),
        MouseDownHit::Segment(segment_hit) => Some(SelItem::Segment {
            shape: segment_hit.shape_id,
            seg: segment_hit.seg_index,
        }),
        MouseDownHit::Shape(ShapeHit { shape_id, .. }) => Some(SelItem::Shape(shape_id)),
        MouseDownHit::None => None,
    }
}

const fn sel_item_for_handle_hit(hit: HandleHit) -> SelItem {
    match hit.kind {
        HandleHitKind::In => SelItem::HandleIn {
            shape: hit.shape_id,
            anchor: hit.anchor_index,
        },
        HandleHitKind::Out => SelItem::HandleOut {
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
