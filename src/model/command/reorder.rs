//! Shape reorder command helpers.

use std::collections::HashSet;

use crate::model::{Document, EngineState, SelItem, Selection, ShapeId};

use super::Command;
use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::ReorderOp;

pub(super) fn prepare_raise_selection(state: &EngineState) -> Result<Command, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    let shape_ids = selected_shape_ids_for_reorder(&state.selection);
    if shape_ids.is_empty() {
        return Err(UserError::EmptySelection);
    }

    for shape_id in &shape_ids {
        if state.document.find_index(*shape_id).is_none() {
            return Err(UserError::ShapeNotFound(*shape_id));
        }
    }

    let operations = reorder_ops(&state.document, &shape_ids, ReorderDirection::Raise);

    if operations.is_empty() {
        return Err(UserError::InvalidOperation("Already at top"));
    }

    Ok(Command::Reorder { operations })
}

pub(super) fn prepare_lower_selection(state: &EngineState) -> Result<Command, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    let shape_ids = selected_shape_ids_for_reorder(&state.selection);
    if shape_ids.is_empty() {
        return Err(UserError::EmptySelection);
    }

    for shape_id in &shape_ids {
        if state.document.find_index(*shape_id).is_none() {
            return Err(UserError::ShapeNotFound(*shape_id));
        }
    }

    let operations = reorder_ops(&state.document, &shape_ids, ReorderDirection::Lower);

    if operations.is_empty() {
        return Err(UserError::InvalidOperation("Already at bottom"));
    }

    Ok(Command::Reorder { operations })
}

#[derive(Clone, Copy, Debug)]
enum ReorderDirection {
    Raise,
    Lower,
}

fn selected_shape_ids_for_reorder(selection: &Selection) -> Vec<ShapeId> {
    let mut seen = HashSet::new();
    let mut shapes = Vec::new();
    for item in &selection.items {
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
    doc: &Document,
    selected: &[ShapeId],
    direction: ReorderDirection,
) -> Vec<ReorderOp> {
    let mut working = doc.clone();
    let mut ops = Vec::new();
    let selected_set: HashSet<ShapeId> = selected.iter().copied().collect();

    if working.shapes.len() < 2 {
        return ops;
    }

    let mut planner = ReorderPlanner {
        working: &mut working,
        selected: &selected_set,
        ops: &mut ops,
    };

    match direction {
        ReorderDirection::Raise => {
            let Some(last_movable) = planner.working.shapes.len().checked_sub(1) else {
                return ops;
            };
            for index in (0..last_movable).rev() {
                planner.try_reorder(index, index + 1);
            }
        }
        ReorderDirection::Lower => {
            let shape_count = planner.working.shapes.len();
            for index in 1..shape_count {
                planner.try_reorder(index, index - 1);
            }
        }
    }

    ops
}

struct ReorderPlanner<'a> {
    working: &'a mut Document,
    selected: &'a HashSet<ShapeId>,
    ops: &'a mut Vec<ReorderOp>,
}

impl ReorderPlanner<'_> {
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

        let op = ReorderOp {
            shape_id: shape.id,
            from_index: from,
            to_index: to,
        };
        let moved_shape = self.working.shapes.remove(from);
        self.working.shapes.insert(to, moved_shape);
        self.ops.push(op);
    }
}

pub(super) fn apply_reorder(
    doc: &mut Document,
    operations: &[ReorderOp],
    command_name: &'static str,
) -> CommandInverse {
    for op in operations {
        if op.from_index == op.to_index {
            continue;
        }
        if op.from_index >= doc.shapes.len() || op.to_index > doc.shapes.len() {
            continue;
        }
        // Verify shape is at expected index
        match doc.find_index(op.shape_id) {
            Some(actual_from) if actual_from == op.from_index => {
                let shape = doc.shapes.remove(op.from_index);
                doc.shapes.insert(op.to_index, shape);
            }
            _ => {}
        }
    }

    // Create inverse with swapped from/to
    let inverse_operations = operations
        .iter()
        .map(|op| ReorderOp {
            shape_id: op.shape_id,
            from_index: op.to_index,
            to_index: op.from_index,
        })
        .collect();

    CommandInverse::ReverseReorder {
        command_name,
        operations: inverse_operations,
    }
}

pub(super) fn apply_reverse_reorder(doc: &mut Document, operations: &[ReorderOp]) {
    // Apply operations in reverse order to correctly undo
    for op in operations.iter().rev() {
        if op.from_index == op.to_index {
            continue;
        }
        if op.from_index >= doc.shapes.len() || op.to_index > doc.shapes.len() {
            continue;
        }
        match doc.find_index(op.shape_id) {
            Some(actual_from) if actual_from == op.from_index => {
                let shape = doc.shapes.remove(op.from_index);
                doc.shapes.insert(op.to_index, shape);
            }
            _ => {}
        }
    }
}
