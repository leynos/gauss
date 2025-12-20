//! Z-order reordering for Phase 0.
//!
//! Phase 0 treats `Document.shapes` order as the render order:
//! later items are painted on top of earlier ones.
//!
//! Raise/lower is implemented as a single-step adjacent move (swap with the
//! next/previous shape) for each selected shape, with adjacent selected shapes
//! moving as a block.

use crate::model::{DocChange, DocOp, SelItem, ShapeId};

use super::{Phase0Shell, draw::ToolMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Raise,
    Lower,
}

impl Phase0Shell {
    pub(super) fn raise_selected_shapes(&mut self) -> bool {
        self.reorder_selected_shapes(Direction::Raise)
    }

    pub(super) fn lower_selected_shapes(&mut self) -> bool {
        self.reorder_selected_shapes(Direction::Lower)
    }

    fn reorder_selected_shapes(&mut self, direction: Direction) -> bool {
        if self.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let selected = selected_shape_ids(&self.selection.items);
        if selected.is_empty() {
            return false;
        }

        let ops = reorder_ops(&self.document, &selected, direction);
        if ops.is_empty() {
            return false;
        }

        self.apply_doc_change(DocChange { ops });
        true
    }
}

fn selected_shape_ids(items: &[SelItem]) -> Vec<ShapeId> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut shapes = Vec::new();
    for item in items {
        let shape = match item {
            SelItem::Shape(shape)
            | SelItem::Anchor { shape, .. }
            | SelItem::HandleIn { shape, .. }
            | SelItem::HandleOut { shape, .. }
            | SelItem::Segment { shape, .. } => *shape,
        };

        if seen.insert(shape) {
            shapes.push(shape);
        }
    }
    shapes
}

fn reorder_ops(
    doc: &crate::model::Document,
    selected: &[ShapeId],
    direction: Direction,
) -> Vec<DocOp> {
    use std::collections::HashSet;

    let mut working = doc.clone();
    let mut ops = Vec::new();
    let selected_set: HashSet<ShapeId> = selected.iter().copied().collect();

    if working.shapes.len() < 2 {
        return ops;
    }

    let mut ctx = ReorderContext {
        working: &mut working,
        selected: &selected_set,
        ops: &mut ops,
    };

    match direction {
        Direction::Raise => {
            let Some(last_movable) = ctx.working.shapes.len().checked_sub(1) else {
                return ops;
            };
            for index in (0..last_movable).rev() {
                ctx.try_reorder(index, index + 1);
            }
        }
        Direction::Lower => {
            for index in 1..ctx.working.shapes.len() {
                ctx.try_reorder(index, index - 1);
            }
        }
    }

    ops
}

struct ReorderContext<'a> {
    working: &'a mut crate::model::Document,
    selected: &'a std::collections::HashSet<ShapeId>,
    ops: &'a mut Vec<DocOp>,
}

impl ReorderContext<'_> {
    fn try_reorder(&mut self, from: usize, to: usize) {
        let Some(shape) = self.working.shapes.get(from) else {
            return;
        };
        if !self.selected.contains(&shape.id) {
            return;
        }

        let Some(other_shape) = self.working.shapes.get(to) else {
            return;
        };
        if self.selected.contains(&other_shape.id) {
            return;
        }

        let op = DocOp::Reorder {
            shape: shape.id,
            from,
            to,
        };
        op.apply(self.working);
        self.ops.push(op);
    }
}
