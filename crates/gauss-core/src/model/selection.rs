//! Selection representation.
//!
//! Selection changes are treated as first-class editor state so they can be
//! undone/redone independently from document edits.

use crate::model::ShapeId;

/// A single selectable item in the editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelItem {
    /// Whole-shape selection.
    Shape(ShapeId),
    /// Anchor point selection.
    Anchor {
        /// Shape containing the anchor.
        shape: ShapeId,
        /// Anchor index within the path.
        anchor: usize,
    },
    /// Incoming handle selection.
    HandleIn {
        /// Shape containing the handle.
        shape: ShapeId,
        /// Anchor index owning the handle.
        anchor: usize,
    },
    /// Outgoing handle selection.
    HandleOut {
        /// Shape containing the handle.
        shape: ShapeId,
        /// Anchor index owning the handle.
        anchor: usize,
    },
    /// Segment selection.
    Segment {
        /// Shape containing the segment.
        shape: ShapeId,
        /// Segment index within the path.
        seg: usize,
    },
}

/// Current selection state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// Selected items. Order is preserved to keep the selection stable.
    pub items: Vec<SelItem>,
}

impl Selection {
    /// Construct an empty selection.
    #[must_use]
    pub const fn empty() -> Self {
        Self { items: Vec::new() }
    }

    /// Return whether the selection is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return whether the selection contains `item`.
    #[must_use]
    pub fn contains(&self, item: &SelItem) -> bool {
        self.items.iter().any(|i| i == item)
    }

    /// Toggle the presence of `item` in the selection.
    ///
    /// If `item` is already selected, it is removed. Otherwise, it is added.
    pub fn toggle(&mut self, item: SelItem) {
        if let Some(pos) = self.items.iter().position(|existing| existing == &item) {
            self.items.remove(pos);
        } else {
            self.items.push(item);
        }
    }

    /// Return an iterator over selected shape IDs.
    ///
    /// Only returns IDs from `SelItem::Shape` variants; anchors, handles, and
    /// segments are filtered out.
    pub fn selected_shapes(&self) -> impl Iterator<Item = ShapeId> + '_ {
        self.items.iter().filter_map(|item| match item {
            SelItem::Shape(id) => Some(*id),
            _ => None,
        })
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::empty()
    }
}
