//! Anchor insertion preparation and segment splitting.

use super::super::{Command, error::UserError, types::ShapeReplacement};
use crate::model::{Anchor, EngineState, PathGeom, SegmentKind, SelItem, Shape, Vec2};

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
            insert_line_midpoint(&mut new_shape.path, seg_index, start_anchor, end_anchor);
        }
        SegmentKind::Cubic => {
            insert_cubic_midpoint(&mut new_shape.path, seg_index, start_anchor, end_anchor)?;
        }
    }

    Some(new_shape)
}

/// Splits a straight segment at its midpoint with a handle-free anchor.
///
/// A line carries no curvature, so the new anchor needs neither handle.
fn insert_line_midpoint(path: &mut PathGeom, seg_index: usize, start: &Anchor, end: &Anchor) {
    let mid = Vec2::new(
        f32::midpoint(start.pos.x, end.pos.x),
        f32::midpoint(start.pos.y, end.pos.y),
    );
    path.anchors.insert(
        seg_index + 1,
        Anchor {
            pos: mid,
            handle_in: None,
            handle_out: None,
        },
    );
    path.segments.insert(seg_index + 1, SegmentKind::Line);
}

/// Splits a cubic segment at t=0.5 using a de Casteljau subdivision.
///
/// The split preserves the original curve exactly: the outer handles of the
/// two halves are re-aimed at the subdivision points so neither half changes
/// shape.
fn insert_cubic_midpoint(
    path: &mut PathGeom,
    seg_index: usize,
    start: &Anchor,
    end: &Anchor,
) -> Option<()> {
    let p0 = start.pos;
    let p1 = start.handle_out.unwrap_or(p0);
    let p2 = end.handle_in.unwrap_or(end.pos);
    let p3 = end.pos;

    let q0 = midpoint(p0, p1);
    let q1 = midpoint(p1, p2);
    let q2 = midpoint(p2, p3);
    let r0 = midpoint(q0, q1);
    let r1 = midpoint(q1, q2);
    let split = midpoint(r0, r1);

    path.anchors.get_mut(seg_index)?.handle_out = Some(q0);
    path.anchors.insert(
        seg_index + 1,
        Anchor {
            pos: split,
            handle_in: Some(r0),
            handle_out: Some(r1),
        },
    );
    // The end anchor moves one place to the right when the split anchor lands.
    path.anchors.get_mut(seg_index + 2)?.handle_in = Some(q2);
    path.segments.insert(seg_index + 1, SegmentKind::Cubic);
    Some(())
}

/// Return the component-wise midpoint of two points.
/// For points `(0, 0)` and `(2, 4)`, the result is `(1, 2)`.
const fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(f32::midpoint(a.x, b.x), f32::midpoint(a.y, b.y))
}
