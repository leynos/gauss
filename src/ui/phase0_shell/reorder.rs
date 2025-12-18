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
    let mut shapes = Vec::new();
    for item in items {
        let shape = match item {
            SelItem::Shape(shape)
            | SelItem::Anchor { shape, .. }
            | SelItem::HandleIn { shape, .. }
            | SelItem::HandleOut { shape, .. }
            | SelItem::Segment { shape, .. } => *shape,
        };

        if shapes.contains(&shape) {
            continue;
        }
        shapes.push(shape);
    }
    shapes
}

fn reorder_ops(
    doc: &crate::model::Document,
    selected: &[ShapeId],
    direction: Direction,
) -> Vec<DocOp> {
    let mut working = doc.clone();
    let mut ops = Vec::new();

    match direction {
        Direction::Raise => {
            let Some(last_movable) = working.shapes.len().checked_sub(1) else {
                return ops;
            };
            for index in (0..last_movable).rev() {
                let Some(shape) = working.shapes.get(index) else {
                    continue;
                };
                if !selected.contains(&shape.id) {
                    continue;
                }

                let next_index = index + 1;
                let Some(next_shape) = working.shapes.get(next_index) else {
                    continue;
                };
                if selected.contains(&next_shape.id) {
                    continue;
                }

                let op = DocOp::Reorder {
                    shape: shape.id,
                    from: index,
                    to: next_index,
                };
                op.apply(&mut working);
                ops.push(op);
            }
        }
        Direction::Lower => {
            for index in 1..working.shapes.len() {
                let Some(shape) = working.shapes.get(index) else {
                    continue;
                };
                if !selected.contains(&shape.id) {
                    continue;
                }

                let prev_index = index - 1;
                let Some(prev_shape) = working.shapes.get(prev_index) else {
                    continue;
                };
                if selected.contains(&prev_shape.id) {
                    continue;
                }

                let op = DocOp::Reorder {
                    shape: shape.id,
                    from: index,
                    to: prev_index,
                };
                op.apply(&mut working);
                ops.push(op);
            }
        }
    }

    ops
}
