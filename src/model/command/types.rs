//! Supporting data structures for commands.

use crate::model::{Anchor, PaintStyle, SegmentKind, Shape, ShapeId, Vec2};

/// A shape that was deleted, with data needed for restoration.
///
/// Captures both the original index (for reinsertion) and the full shape
/// data (for restoration on undo).
#[derive(Clone, Debug, PartialEq)]
pub struct DeletedShape {
    /// Original index in the document's shape list.
    pub index: usize,
    /// The deleted shape data.
    pub shape: Shape,
}

/// A shape insertion with data needed for undo.
///
/// Captures the insertion index and the shape data for both
/// application and reversal on undo.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeInsertion {
    /// Index to insert at in the document's shape list.
    pub index: usize,
    /// The shape to insert.
    pub shape: Shape,
}

/// A style change for a single shape.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleChange {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Previous style.
    pub from: PaintStyle,
    /// New style.
    pub to: PaintStyle,
}

/// A reorder operation for a single shape.
#[derive(Clone, Debug, PartialEq)]
pub struct ReorderOp {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Source index in the document's shape list.
    pub from_index: usize,
    /// Destination index in the document's shape list.
    pub to_index: usize,
}

/// A segment kind change, including handle adjustments.
///
/// When toggling between Line and Cubic segments, handles must be
/// created or removed. This structure captures all data needed for
/// both the forward operation and undo.
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentChange {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Segment index within the path.
    pub segment_index: usize,
    /// Previous segment kind.
    pub old_kind: SegmentKind,
    /// New segment kind.
    pub new_kind: SegmentKind,
    /// Previous outgoing handle of the start anchor.
    pub old_start_handle_out: Option<Vec2>,
    /// New outgoing handle of the start anchor.
    pub new_start_handle_out: Option<Vec2>,
    /// Previous incoming handle of the end anchor.
    pub old_end_handle_in: Option<Vec2>,
    /// New incoming handle of the end anchor.
    pub new_end_handle_in: Option<Vec2>,
}

/// A shape movement (translation by delta).
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeMovement {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Translation delta.
    pub delta: Vec2,
}

/// Which handle of an anchor (incoming or outgoing).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleKind {
    /// The incoming handle (`handle_in`).
    In,
    /// The outgoing handle (`handle_out`).
    Out,
}

/// Result of deleting anchors from a shape.
#[derive(Clone, Debug, PartialEq)]
pub enum AnchorDeletionResult {
    /// Shape was modified (still has >= 2 anchors).
    Modified(Shape),
    /// Shape was entirely removed (would have < 2 anchors).
    Removed,
}

/// Data for restoring a shape after anchor deletion.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorDeletion {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Shape index in the document.
    pub shape_index: usize,
    /// Original shape before deletion.
    pub old_shape: Shape,
    /// Result of the deletion operation.
    pub result: AnchorDeletionResult,
}

/// Data for restoring anchors (inverse of deletion).
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorRestoration {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Shape index in the document.
    pub shape_index: usize,
    /// How to restore the shape.
    pub restoration: AnchorRestorationKind,
}

/// How to restore a shape after anchor deletion was undone.
#[derive(Clone, Debug, PartialEq)]
pub enum AnchorRestorationKind {
    /// Replace the modified shape with the original.
    RestoreModified {
        /// Shape after anchor deletion (current state).
        from: Shape,
        /// Original shape before deletion (to restore).
        to: Shape,
    },
    /// Re-insert a shape that was entirely removed.
    RestoreRemoved {
        /// The removed shape to restore.
        shape: Shape,
    },
}

/// Data for an anchor movement, capturing the original anchor state.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorMovement {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Anchor index within the path.
    pub anchor_index: usize,
    /// Original anchor state (position and handles).
    pub original: Anchor,
    /// Translation delta applied.
    pub delta: Vec2,
}

/// Data for a handle movement.
#[derive(Clone, Debug, PartialEq)]
pub struct HandleMovement {
    /// Target shape identifier.
    pub shape_id: ShapeId,
    /// Anchor index within the path.
    pub anchor_index: usize,
    /// Which handle was moved.
    pub kind: HandleKind,
    /// Previous handle position.
    pub from: Option<Vec2>,
    /// New handle position.
    pub to: Option<Vec2>,
}

/// Data for a shape replacement (used by `InsertAnchor`,
/// `InsertAnchorOnSegment`, `ClosePath`).
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeReplacement {
    /// Shape index in the document.
    pub shape_index: usize,
    /// Original shape before modification.
    pub old_shape: Shape,
    /// New shape after modification.
    pub new_shape: Shape,
}
