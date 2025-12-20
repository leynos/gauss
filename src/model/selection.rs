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

    /// Return whether the selection contains `item`.
    #[must_use]
    pub fn contains(&self, item: &SelItem) -> bool {
        self.items.iter().any(|i| i == item)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::empty()
    }
}
