//! Shared selection helpers for Phase 0 UI modules.

use crate::model::{SelItem, Selection, ShapeId};

/// Yields IDs for selected shapes, skipping selected anchors, handles, and segments.
///
/// For a selection containing shape `A` and an anchor on shape `B`, the iterator
/// yields only `A`; callers can then retrieve those shapes from the document.
pub(crate) fn selected_shape_ids(selection: &Selection) -> impl Iterator<Item = ShapeId> + '_ {
    selection.items.iter().filter_map(|item| match item {
        SelItem::Shape(id) => Some(*id),
        _ => None,
    })
}
