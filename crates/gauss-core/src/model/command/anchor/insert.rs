//! Anchor insertion preparation and segment splitting.

use super::super::{Command, error::UserError, types::ShapeReplacement};
use crate::model::{Anchor, EngineState, SegmentKind, SelItem, Shape, Vec2};

/// Prepare an insertion command while retaining both shapes for undo.
/// A selected segment yields a command whose inverse restores its original path.
pub(in crate::model::command) fn prepare_insert_anchor_on_segment(
    state: &EngineState,
) -> Result<Command, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    // Find selected segment
    let seg_selection = state
        .selection
        .items
        .iter()
        .find(|item| matches!(item, SelItem::Segment { .. }));

    let Some(SelItem::Segment { shape, seg }) = seg_selection else {
        return Err(UserError::InvalidOperation("No segment selected".into()));
    };

    let Some(shape_index) = state.document.find_index(*shape) else {
        return Err(UserError::ShapeNotFound(*shape));
    };

    let Some(old_shape) = state.document.shape_at(shape_index).cloned() else {
        return Err(UserError::ShapeNotFound(*shape));
    };

    // Create new shape with inserted anchor
    let Some(new_shape) = insert_anchor_into_shape(&old_shape, *seg) else {
        return Err(UserError::InvalidOperation(
            "Cannot insert anchor on segment".into(),
        ));
    };

    Ok(Command::InsertAnchorOnSegment {
        replacement: ShapeReplacement {
            shape_index,
            old_shape,
            new_shape,
        },
    })
}

/// Insert an anchor at the midpoint of a segment.
fn insert_anchor_into_shape(shape: &Shape, seg_index: usize) -> Option<Shape> {
    let path = &shape.path;
    let seg_kind = *path.segments.get(seg_index)?;
    let start_anchor = path.anchors.get(seg_index)?;
    let end_anchor = path.anchors.get(seg_index + 1)?;

    let mut new_shape = shape.clone();

    match seg_kind {
        SegmentKind::Line => {
            // For line segments, insert midpoint with no handles
            let mid = Vec2::new(
                f32::midpoint(start_anchor.pos.x, end_anchor.pos.x),
                f32::midpoint(start_anchor.pos.y, end_anchor.pos.y),
            );
            let new_anchor = Anchor {
                pos: mid,
                handle_in: None,
                handle_out: None,
            };
            new_shape.path.anchors.insert(seg_index + 1, new_anchor);
            new_shape
                .path
                .segments
                .insert(seg_index + 1, SegmentKind::Line);
        }
        SegmentKind::Cubic => {
            // For cubic segments, use de Casteljau split at t=0.5
            let p0 = start_anchor.pos;
            let p1 = start_anchor.handle_out.unwrap_or(p0);
            let p2 = end_anchor.handle_in.unwrap_or(end_anchor.pos);
            let p3 = end_anchor.pos;

            // De Casteljau at t=0.5
            let q0 = midpoint(p0, p1);
            let q1 = midpoint(p1, p2);
            let q2 = midpoint(p2, p3);
            let r0 = midpoint(q0, q1);
            let r1 = midpoint(q1, q2);
            let s = midpoint(r0, r1);

            // Update start anchor's handle_out
            new_shape.path.anchors.get_mut(seg_index)?.handle_out = Some(q0);

            // Insert new anchor at split point
            let new_anchor = Anchor {
                pos: s,
                handle_in: Some(r0),
                handle_out: Some(r1),
            };
            new_shape.path.anchors.insert(seg_index + 1, new_anchor);

            // Update end anchor's handle_in (now at seg_index + 2 after insert)
            new_shape.path.anchors.get_mut(seg_index + 2)?.handle_in = Some(q2);

            // Insert new segment
            new_shape
                .path
                .segments
                .insert(seg_index + 1, SegmentKind::Cubic);
        }
    }

    Some(new_shape)
}

/// Return the component-wise midpoint used when splitting a cubic segment.
/// For points `(0, 0)` and `(2, 4)`, the result is `(1, 2)`.
const fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(f32::midpoint(a.x, b.x), f32::midpoint(a.y, b.y))
}
