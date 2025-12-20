//! Shared selection helpers for Phase 0 UI modules.

use crate::model::{SelItem, Selection, ShapeId};

pub(crate) fn selected_shape_ids(selection: &Selection) -> impl Iterator<Item = ShapeId> + '_ {
    selection.items.iter().filter_map(|item| match item {
        SelItem::Shape(id) => Some(*id),
        _ => None,
    })
}
