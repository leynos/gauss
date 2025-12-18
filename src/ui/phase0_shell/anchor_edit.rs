//! PoC-quality anchor insertion and deletion for Phase 0.
//!
//! Phase 0 aims to validate editor workflows without committing to a fully
//! general structural editing engine. For that reason, we encode anchor edits
//! as “replace the entire shape” document changes. This keeps the operations
//! trivially invertible and avoids implementing a complex patch format early.

use crate::model::{
    Anchor, DocChange, DocOp, SegmentKind, SelItem, Selection, Shape, ShapeId, Vec2,
};

use super::{Phase0Shell, draw::ToolMode};

impl Phase0Shell {
    pub(super) fn insert_anchor_on_selected_segment(&mut self) -> bool {
        if self.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let Some((shape_id, seg_index)) = first_selected_segment(&self.selection.items) else {
            return false;
        };

        let mut working = self.document.clone();
        let Some(index) = working.find_index(shape_id) else {
            return false;
        };
        let Some(old_shape) = working.shapes.get(index).cloned() else {
            return false;
        };

        let Some(new_shape) = insert_anchor_into_shape(old_shape.clone(), seg_index) else {
            return false;
        };

        let mut ops = Vec::new();
        let mut edit = DocEditContext::new(&mut working, &mut ops);
        edit.replace_shape(index, old_shape, new_shape);
        if ops.is_empty() {
            return false;
        }

        self.apply_doc_change(DocChange { ops });

        let previous_selection = self.selection.clone();
        let new_selection = Selection {
            items: vec![SelItem::Anchor {
                shape: shape_id,
                anchor: seg_index + 1,
            }],
        };
        self.record_selection_change(previous_selection, new_selection.clone());
        self.selection = new_selection;
        self.drag_state = None;
        true
    }

    pub(super) fn delete_selected_anchors(&mut self) -> bool {
        if self.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let grouped = group_selected_anchors(&self.selection.items);
        if grouped.is_empty() {
            return false;
        }

        let mut working = self.document.clone();
        let mut ops = Vec::new();
        let mut edit = DocEditContext::new(&mut working, &mut ops);

        for (shape_id, anchor_indices) in grouped {
            apply_anchor_deletion_for_shape(&mut edit, shape_id, &anchor_indices);
        }

        if ops.is_empty() {
            return false;
        }

        self.apply_doc_change(DocChange { ops });

        let previous_selection = self.selection.clone();
        let new_selection = Selection::empty();
        self.record_selection_change(previous_selection, new_selection.clone());
        self.selection = new_selection;
        self.drag_state = None;
        true
    }
}

fn first_selected_segment(items: &[SelItem]) -> Option<(ShapeId, usize)> {
    for item in items {
        if let SelItem::Segment { shape, seg } = item {
            return Some((*shape, *seg));
        }
    }
    None
}

fn group_selected_anchors(items: &[SelItem]) -> Vec<(ShapeId, Vec<usize>)> {
    let mut grouped = Vec::<(ShapeId, Vec<usize>)>::new();
    for item in items {
        let SelItem::Anchor { shape, anchor } = item else {
            continue;
        };

        let Some(existing) = grouped.iter_mut().find(|(id, _)| id == shape) else {
            grouped.push((*shape, vec![*anchor]));
            continue;
        };

        if existing.1.contains(anchor) {
            continue;
        }
        existing.1.push(*anchor);
    }

    for (_, indices) in &mut grouped {
        indices.sort_unstable_by(|a, b| b.cmp(a));
    }

    grouped
}

fn apply_anchor_deletion_for_shape(
    edit: &mut DocEditContext<'_>,
    shape_id: ShapeId,
    anchor_indices: &[usize],
) {
    let Some(index) = edit.working.find_index(shape_id) else {
        return;
    };
    let Some(old_shape) = edit.working.shapes.get(index).cloned() else {
        return;
    };

    match delete_anchors_from_shape(old_shape.clone(), anchor_indices) {
        None => {
            let remove = DocOp::RemoveShape {
                index,
                shape: old_shape,
            };
            remove.apply(edit.working);
            edit.ops.push(remove);
        }
        Some(new_shape) => edit.replace_shape(index, old_shape, new_shape),
    }
}

struct DocEditContext<'a> {
    working: &'a mut crate::model::Document,
    ops: &'a mut Vec<DocOp>,
}

impl<'a> DocEditContext<'a> {
    const fn new(working: &'a mut crate::model::Document, ops: &'a mut Vec<DocOp>) -> Self {
        Self { working, ops }
    }

    fn replace_shape(&mut self, index: usize, old_shape: Shape, new_shape: Shape) {
        let remove = DocOp::RemoveShape {
            index,
            shape: old_shape,
        };
        remove.apply(self.working);
        self.ops.push(remove);

        let insert = DocOp::InsertShape {
            index,
            shape: new_shape,
        };
        insert.apply(self.working);
        self.ops.push(insert);
    }
}

fn delete_anchors_from_shape(mut shape: Shape, anchor_indices: &[usize]) -> Option<Shape> {
    for anchor_index in anchor_indices {
        if *anchor_index >= shape.path.anchors.len() {
            continue;
        }
        shape.path.anchors.remove(*anchor_index);
    }

    if shape.path.anchors.len() < 2 {
        return None;
    }

    normalize_path_as_polyline(&mut shape);
    Some(shape)
}

fn normalize_path_as_polyline(shape: &mut Shape) {
    for anchor in &mut shape.path.anchors {
        anchor.handle_in = None;
        anchor.handle_out = None;
    }

    shape.path.segments = std::iter::repeat_n(
        SegmentKind::Line,
        shape.path.anchors.len().saturating_sub(1),
    )
    .collect();

    if shape.path.closed && shape.path.anchors.len() < 3 {
        shape.path.closed = false;
    }
}

fn insert_anchor_into_shape(mut shape: Shape, seg_index: usize) -> Option<Shape> {
    let kind = shape.path.segments.get(seg_index).copied()?;
    let start_anchor = shape.path.anchors.get(seg_index).cloned()?;
    let end_anchor = shape.path.anchors.get(seg_index + 1).cloned()?;

    let (new_anchor, new_start_handle_out, new_end_handle_in) = match kind {
        SegmentKind::Line => {
            let mid = start_anchor.pos.add(end_anchor.pos).mul(0.5);
            (Anchor::new(mid), None, None)
        }
        SegmentKind::Cubic => split_cubic_at_half(&start_anchor, &end_anchor),
    };

    shape.path.anchors.insert(seg_index + 1, new_anchor);
    shape.path.segments.insert(seg_index + 1, kind);

    set_anchor_handle_out(&mut shape, seg_index, new_start_handle_out);
    set_anchor_handle_in(&mut shape, seg_index + 2, new_end_handle_in);

    Some(shape)
}

fn set_anchor_handle_out(shape: &mut Shape, index: usize, handle: Option<Vec2>) {
    if let Some(anchor) = shape.path.anchors.get_mut(index) {
        anchor.handle_out = handle;
    }
}

fn set_anchor_handle_in(shape: &mut Shape, index: usize, handle: Option<Vec2>) {
    if let Some(anchor) = shape.path.anchors.get_mut(index) {
        anchor.handle_in = handle;
    }
}

fn split_cubic_at_half(start: &Anchor, end: &Anchor) -> (Anchor, Option<Vec2>, Option<Vec2>) {
    let p0 = start.pos;
    let p1 = start.handle_out.unwrap_or(start.pos);
    let p2 = end.handle_in.unwrap_or(end.pos);
    let p3 = end.pos;

    let q0 = lerp(p0, p1);
    let q1 = lerp(p1, p2);
    let q2 = lerp(p2, p3);
    let r0 = lerp(q0, q1);
    let r1 = lerp(q1, q2);
    let s = lerp(r0, r1);

    let new_anchor = Anchor {
        pos: s,
        handle_in: Some(r0),
        handle_out: Some(r1),
    };

    (new_anchor, Some(q0), Some(q2))
}

const fn lerp(a: Vec2, b: Vec2) -> Vec2 {
    a.add(b).mul(0.5)
}
