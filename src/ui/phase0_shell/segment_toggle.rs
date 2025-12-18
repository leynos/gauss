//! Segment kind toggling for Phase 0.
//!
//! In manipulate mode, the user can select a segment and toggle it between a
//! straight line and a cubic Bézier. This is intentionally a "`PoC` quality"
//! operation: we synthesise initial handles for new cubic segments and clear
//! the handles when converting back to lines.

use crate::model::{DocChange, DocOp, SegmentKind, SelItem, ShapeId, Vec2};

use super::Phase0Shell;

impl Phase0Shell {
    pub(super) fn toggle_selected_segments_kind(&mut self) -> bool {
        let segments = selected_segments(&self.selection.items);
        if segments.is_empty() {
            return false;
        }

        let mut ops = Vec::new();
        for (shape_id, seg_index) in segments {
            append_segment_toggle_ops(&self.document, shape_id, seg_index, &mut ops);
        }

        if ops.is_empty() {
            return false;
        }

        self.apply_doc_change(DocChange { ops });
        true
    }
}

fn selected_segments(items: &[SelItem]) -> Vec<(ShapeId, usize)> {
    let mut segments = Vec::new();
    for item in items {
        let SelItem::Segment { shape, seg } = item else {
            continue;
        };
        if segments.contains(&(*shape, *seg)) {
            continue;
        }
        segments.push((*shape, *seg));
    }
    segments
}

fn append_segment_toggle_ops(
    doc: &crate::model::Document,
    shape_id: ShapeId,
    seg_index: usize,
    ops: &mut Vec<DocOp>,
) {
    let Some(shape_index) = doc.find_index(shape_id) else {
        return;
    };
    let Some(shape) = doc.shapes.get(shape_index) else {
        return;
    };

    let Some(kind) = shape.path.segments.get(seg_index) else {
        return;
    };

    let Some(start_anchor) = shape.path.anchors.get(seg_index) else {
        return;
    };
    let Some(end_anchor) = shape.path.anchors.get(seg_index + 1) else {
        return;
    };

    match kind {
        SegmentKind::Line => {
            let (handle_out, handle_in) = line_to_cubic_handles(start_anchor.pos, end_anchor.pos);

            ops.push(DocOp::MoveHandleOut {
                shape: shape_id,
                anchor: seg_index,
                from: start_anchor.handle_out,
                to: Some(handle_out),
            });
            ops.push(DocOp::MoveHandleIn {
                shape: shape_id,
                anchor: seg_index + 1,
                from: end_anchor.handle_in,
                to: Some(handle_in),
            });
            ops.push(DocOp::SetSegmentKind {
                shape: shape_id,
                seg: seg_index,
                from: *kind,
                to: SegmentKind::Cubic,
            });
        }
        SegmentKind::Cubic => {
            ops.push(DocOp::MoveHandleOut {
                shape: shape_id,
                anchor: seg_index,
                from: start_anchor.handle_out,
                to: None,
            });
            ops.push(DocOp::MoveHandleIn {
                shape: shape_id,
                anchor: seg_index + 1,
                from: end_anchor.handle_in,
                to: None,
            });
            ops.push(DocOp::SetSegmentKind {
                shape: shape_id,
                seg: seg_index,
                from: *kind,
                to: SegmentKind::Line,
            });
        }
    }
}

fn line_to_cubic_handles(start: Vec2, end: Vec2) -> (Vec2, Vec2) {
    let delta = end.sub(start);
    let third = delta.mul(1.0 / 3.0);
    let handle_out = start.add(third);
    let handle_in = end.sub(third);
    (handle_out, handle_in)
}
