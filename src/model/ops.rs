//! Invertible document and selection operations for undo/redo.

#![expect(
    clippy::float_arithmetic,
    reason = "document operations use vector maths for transforms"
)]

use crate::model::{Document, PaintStyle, SegmentKind, Shape, ShapeId, Vec2};

/// A single invertible document operation.
#[derive(Clone, Debug, PartialEq)]
pub enum DocOp {
    /// Insert a shape at a specific index.
    InsertShape {
        /// Index to insert at.
        index: usize,
        /// Shape to insert.
        shape: Shape,
    },
    /// Remove a shape at a specific index.
    RemoveShape {
        /// Index to remove from.
        index: usize,
        /// Removed shape (retained for inversion).
        shape: Shape,
    },

    /// Move an anchor point.
    MoveAnchor {
        /// Target shape.
        shape: ShapeId,
        /// Anchor index within the path.
        anchor: usize,
        /// Previous position.
        from: Vec2,
        /// New position.
        to: Vec2,
    },

    /// Update the incoming handle of an anchor.
    MoveHandleIn {
        /// Target shape.
        shape: ShapeId,
        /// Anchor index within the path.
        anchor: usize,
        /// Previous handle position.
        from: Option<Vec2>,
        /// New handle position.
        to: Option<Vec2>,
    },

    /// Update the outgoing handle of an anchor.
    MoveHandleOut {
        /// Target shape.
        shape: ShapeId,
        /// Anchor index within the path.
        anchor: usize,
        /// Previous handle position.
        from: Option<Vec2>,
        /// New handle position.
        to: Option<Vec2>,
    },

    /// Move an entire shape by a delta.
    MoveShape {
        /// Target shape.
        shape: ShapeId,
        /// Translation delta.
        delta: Vec2,
    },

    /// Replace a shape's style.
    SetStyle {
        /// Target shape.
        shape: ShapeId,
        /// Previous style.
        from: PaintStyle,
        /// New style.
        to: PaintStyle,
    },

    /// Update a segment kind.
    SetSegmentKind {
        /// Target shape.
        shape: ShapeId,
        /// Segment index.
        seg: usize,
        /// Previous kind.
        from: SegmentKind,
        /// New kind.
        to: SegmentKind,
    },

    /// Reorder a shape within the document.
    Reorder {
        /// Target shape.
        shape: ShapeId,
        /// Source index.
        from: usize,
        /// Destination index.
        to: usize,
    },
}

impl DocOp {
    /// Construct the inverse operation.
    #[must_use]
    pub fn invert(&self) -> Self {
        match self {
            Self::InsertShape { index, shape } => Self::RemoveShape {
                index: *index,
                shape: shape.clone(),
            },
            Self::RemoveShape { index, shape } => Self::InsertShape {
                index: *index,
                shape: shape.clone(),
            },
            Self::MoveAnchor {
                shape,
                anchor,
                from,
                to,
            } => Self::MoveAnchor {
                shape: *shape,
                anchor: *anchor,
                from: *to,
                to: *from,
            },
            Self::MoveHandleIn {
                shape,
                anchor,
                from,
                to,
            } => Self::MoveHandleIn {
                shape: *shape,
                anchor: *anchor,
                from: *to,
                to: *from,
            },
            Self::MoveHandleOut {
                shape,
                anchor,
                from,
                to,
            } => Self::MoveHandleOut {
                shape: *shape,
                anchor: *anchor,
                from: *to,
                to: *from,
            },
            Self::MoveShape { shape, delta } => Self::MoveShape {
                shape: *shape,
                delta: Vec2::new(-delta.x, -delta.y),
            },
            Self::SetStyle { shape, from, to } => Self::SetStyle {
                shape: *shape,
                from: to.clone(),
                to: from.clone(),
            },
            Self::SetSegmentKind {
                shape,
                seg,
                from,
                to,
            } => Self::SetSegmentKind {
                shape: *shape,
                seg: *seg,
                from: *to,
                to: *from,
            },
            Self::Reorder { shape, from, to } => Self::Reorder {
                shape: *shape,
                from: *to,
                to: *from,
            },
        }
    }

    /// Apply the operation to the document.
    ///
    /// Invalid references (missing shapes, out-of-range indices) are ignored in
    /// this `PoC` layer. Controller code should construct valid operations.
    pub fn apply(&self, doc: &mut Document) {
        match self {
            Self::InsertShape { index, shape } => apply_insert_shape(doc, *index, shape),
            Self::RemoveShape { index, .. } => apply_remove_shape(doc, *index),
            Self::MoveAnchor {
                shape, anchor, to, ..
            } => apply_move_anchor(doc, *shape, *anchor, *to),
            Self::MoveHandleIn {
                shape, anchor, to, ..
            } => {
                apply_move_handle_in(doc, *shape, *anchor, *to);
            }
            Self::MoveHandleOut {
                shape, anchor, to, ..
            } => {
                apply_move_handle_out(doc, *shape, *anchor, *to);
            }
            Self::MoveShape { shape, delta } => apply_move_shape(doc, *shape, *delta),
            Self::SetStyle { shape, to, .. } => apply_set_style(doc, *shape, to),
            Self::SetSegmentKind { shape, seg, to, .. } => {
                apply_set_segment_kind(doc, *shape, *seg, *to);
            }
            Self::Reorder { shape, from, to } => apply_reorder(doc, *shape, *from, *to),
        }
    }
}

fn apply_insert_shape(doc: &mut Document, index: usize, shape: &Shape) {
    let _ = doc.insert_shape(index, shape.clone());
}

fn apply_remove_shape(doc: &mut Document, index: usize) {
    let _ = doc.remove_shape(index);
}

fn apply_move_anchor(doc: &mut Document, shape: ShapeId, anchor: usize, to: Vec2) {
    let Some(target) = doc.shape_mut(shape) else {
        return;
    };
    let Some(item) = target.path.anchors.get_mut(anchor) else {
        return;
    };
    item.pos = to;
}

fn apply_move_handle_in(doc: &mut Document, shape: ShapeId, anchor: usize, to: Option<Vec2>) {
    let Some(target) = doc.shape_mut(shape) else {
        return;
    };
    let Some(item) = target.path.anchors.get_mut(anchor) else {
        return;
    };
    item.handle_in = to;
}

fn apply_move_handle_out(doc: &mut Document, shape: ShapeId, anchor: usize, to: Option<Vec2>) {
    let Some(target) = doc.shape_mut(shape) else {
        return;
    };
    let Some(item) = target.path.anchors.get_mut(anchor) else {
        return;
    };
    item.handle_out = to;
}

fn apply_move_shape(doc: &mut Document, shape: ShapeId, delta: Vec2) {
    let Some(target) = doc.shape_mut(shape) else {
        return;
    };

    for anchor in &mut target.path.anchors {
        translate_anchor(anchor, delta);
    }
}

#[expect(
    clippy::missing_const_for_fn,
    reason = "anchor mutation is runtime-only and clearer as a non-const helper"
)]
fn translate_anchor(anchor: &mut crate::model::Anchor, delta: Vec2) {
    anchor.pos = anchor.pos.add(delta);

    if let Some(handle) = anchor.handle_in {
        anchor.handle_in = Some(handle.add(delta));
    }
    if let Some(handle) = anchor.handle_out {
        anchor.handle_out = Some(handle.add(delta));
    }
}

fn apply_set_style(doc: &mut Document, shape: ShapeId, to: &PaintStyle) {
    let Some(target) = doc.shape_mut(shape) else {
        return;
    };
    target.style = to.clone();
}

fn apply_set_segment_kind(doc: &mut Document, shape: ShapeId, seg: usize, to: SegmentKind) {
    let Some(target) = doc.shape_mut(shape) else {
        return;
    };
    let Some(kind) = target.path.segments.get_mut(seg) else {
        return;
    };
    *kind = to;
}

fn apply_reorder(doc: &mut Document, shape: ShapeId, from: usize, to: usize) {
    if from == to {
        return;
    }

    if doc.len() <= from || to > doc.len() {
        return;
    }

    let Some(actual_from) = doc.find_index(shape) else {
        return;
    };
    if actual_from != from {
        return;
    }
    let _ = doc.reorder(from, to);
}

/// A group of operations treated as one undo/redo unit.
#[derive(Clone, Debug, PartialEq)]
pub struct DocChange {
    /// Operations in forward order.
    pub ops: Vec<DocOp>,
}

impl DocChange {
    /// Apply the change to the document.
    pub fn apply(&self, doc: &mut Document) {
        for op in &self.ops {
            op.apply(doc);
        }
    }

    /// Apply the inverse change to the document.
    pub fn apply_inverse(&self, doc: &mut Document) {
        for op in self.ops.iter().rev() {
            op.invert().apply(doc);
        }
    }
}
