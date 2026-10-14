//! Shape reorder command helpers.

use std::collections::HashSet;

use crate::model::{Document, EngineState, SelItem, Selection, ShapeId};

use super::Command;
use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::ReorderOp;

/// Build reorder operations for the selected shapes in the requested direction.
///
/// # Errors
///
/// Returns an error for an empty selection, missing shape, or selection already
/// at the requested document boundary.
/// A movable selection returns `Command::Reorder` with its adjacent moves.
fn prepare_reorder_selection(
    state: &EngineState,
    direction: ReorderDirection,
    at_limit_msg: &'static str,
) -> Result<Command, UserError> {
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

    let operations = reorder_ops(&state.document, &shape_ids, direction);

    if operations.is_empty() {
        return Err(UserError::InvalidOperation(at_limit_msg.into()));
    }

    Ok(Command::Reorder { operations })
}

/// Prepare adjacent reorder operations for selected shapes that can move one
/// layer in the requested direction.
///
/// Selected neighbours and boundary positions may produce no operation for a
/// shape. For example, raising `[A, B, C]` with A and C selected can move A past
/// B while C remains at the top.
///
/// # Errors
///
/// Returns an error when no shape can be raised or a selected shape is missing.
pub(super) fn prepare_raise_selection(state: &EngineState) -> Result<Command, UserError> {
    prepare_reorder_selection(state, ReorderDirection::Raise, "Already at top")
}

/// Prepare adjacent reorder operations for selected shapes that can move one
/// layer in the requested direction.
///
/// Selected neighbours and boundary positions may produce no operation for a
/// shape. For example, lowering `[A, B, C]` with A and C selected can move C
/// past B while A remains at the bottom.
///
/// # Errors
///
/// Returns an error when no shape can be lowered or a selected shape is missing.
pub(super) fn prepare_lower_selection(state: &EngineState) -> Result<Command, UserError> {
    prepare_reorder_selection(state, ReorderDirection::Lower, "Already at bottom")
}

/// Direction in which selected shapes move through the document's stacking order.
#[derive(Clone, Copy, Debug)]
enum ReorderDirection {
    /// Move selected shapes towards the end of the document.
    Raise,
    /// Move selected shapes towards the start of the document.
    Lower,
}

/// Collect selected shape IDs once, preserving their first-seen order.
/// Selecting the same shape by both anchor and segment yields one ID.
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

/// Plan adjacent moves for selected shapes without mutating the source document.
/// A selected shape at index 0 can produce a move to index 1.
fn reorder_ops(
    doc: &Document,
    selected: &[ShapeId],
    direction: ReorderDirection,
) -> Vec<ReorderOp> {
    if doc.len() < 2 {
        return Vec::new();
    }

    let mut working = doc.clone();
    let mut ops = Vec::new();
    let selected_set: HashSet<ShapeId> = selected.iter().copied().collect();

    let mut planner = ReorderPlanner {
        working: &mut working,
        selected: &selected_set,
        ops: &mut ops,
    };

    match direction {
        ReorderDirection::Raise => reorder_raise(&mut planner),
        ReorderDirection::Lower => reorder_lower(&mut planner),
    }

    ops
}

/// Walk backwards so each selected shape moves up one position without skipping peers.
/// In `[A, B, C]`, selecting `A` plans a move in front of `B`.
fn reorder_raise(planner: &mut ReorderPlanner) {
    let Some(last_movable) = planner.working.len().checked_sub(1) else {
        return;
    };
    for index in (0..last_movable).rev() {
        if !planner.try_reorder(index, index + 1) {
            break;
        }
    }
}

/// Walk forwards so each selected shape moves down one position without skipping peers.
/// In `[A, B, C]`, selecting `C` plans a move before `B`.
fn reorder_lower(planner: &mut ReorderPlanner) {
    let shape_count = planner.working.len();
    for index in 1..shape_count {
        if !planner.try_reorder(index, index - 1) {
            break;
        }
    }
}

/// Applies planned adjacent moves to a working copy and records successful operations.
struct ReorderPlanner<'a> {
    /// Mutable document copy whose order is used to plan subsequent moves.
    working: &'a mut Document,
    /// Shape IDs that may move across unselected neighbours.
    selected: &'a HashSet<ShapeId>,
    /// Operations emitted in the order they must be applied.
    ops: &'a mut Vec<ReorderOp>,
}

impl ReorderPlanner<'_> {
    /// Move a selected shape by one position, or report that planning must stop.
    /// A selected `from` shape crossing an unselected `to` shape records one move.
    fn try_reorder(&mut self, from: usize, to: usize) -> bool {
        let Some(shape) = self.working.shape_at(from) else {
            return false;
        };
        if !self.selected.contains(&shape.id) {
            return true;
        }

        let Some(other_shape) = self.working.shape_at(to) else {
            return false;
        };
        if self.selected.contains(&other_shape.id) {
            return true;
        }

        let op = ReorderOp {
            shape_id: shape.id,
            from_index: from,
            to_index: to,
        };
        if self.working.reorder(from, to) {
            self.ops.push(op);
            return true;
        }
        false
    }
}

/// Check that an operation changes the order and both indices address the document.
/// In a three-shape document, a move from index 1 to 2 passes this check.
fn is_valid_reorder_op(op: &ReorderOp, doc: &Document) -> bool {
    if op.from_index == op.to_index {
        return false;
    }

    op.from_index < doc.len() && op.to_index < doc.len()
}

/// Apply an operation only when its shape still occupies the expected source index.
/// If that shape moved since planning, this helper leaves the document unchanged.
fn try_move_shape(doc: &mut Document, op: &ReorderOp) -> bool {
    match doc.find_index(op.shape_id) {
        Some(actual_from) if actual_from == op.from_index => {
            doc.reorder(op.from_index, op.to_index)
        }
        _ => false,
    }
}

/// Apply reorder operations, returning the inverse command for undo.
///
/// # Errors
///
/// Returns `UserError::InvalidOperation` if any reorder operation is invalid.
/// Returns `UserError::ShapeNotFound` if a shape is not at the expected index.
pub(super) fn apply_reorder(
    doc: &mut Document,
    operations: &[ReorderOp],
    command_name: &'static str,
) -> Result<CommandInverse, UserError> {
    for op in operations {
        if !is_valid_reorder_op(op, doc) {
            return Err(UserError::InvalidOperation(format!(
                "invalid reorder: from {} to {} (doc len = {})",
                op.from_index,
                op.to_index,
                doc.len()
            )));
        }
        if !try_move_shape(doc, op) {
            return Err(UserError::ShapeNotFound(op.shape_id));
        }
    }

    // Create inverse with swapped from/to
    let inverse_operations = create_inverse_operations(operations);

    Ok(CommandInverse::ReverseReorder {
        command_name,
        operations: inverse_operations,
    })
}

/// Reverse each operation's source and destination indices for undo.
/// An operation from index 1 to 2 becomes one from 2 to 1.
fn create_inverse_operations(operations: &[ReorderOp]) -> Vec<ReorderOp> {
    operations
        .iter()
        .map(|op| ReorderOp {
            shape_id: op.shape_id,
            from_index: op.to_index,
            to_index: op.from_index,
        })
        .collect()
}

/// Apply the inverse of reorder operations (for undo).
///
/// # Errors
///
/// Returns `UserError::InvalidOperation` if any reorder operation is invalid.
/// Returns `UserError::ShapeNotFound` if a shape is not at the expected index.
pub(super) fn apply_reverse_reorder(
    doc: &mut Document,
    operations: &[ReorderOp],
) -> Result<(), UserError> {
    // Apply operations in reverse order to correctly undo
    for op in operations.iter().rev() {
        if !is_valid_reorder_op(op, doc) {
            return Err(UserError::InvalidOperation(format!(
                "invalid reverse reorder: from {} to {} (doc len = {})",
                op.from_index,
                op.to_index,
                doc.len()
            )));
        }
        if !try_move_shape(doc, op) {
            return Err(UserError::ShapeNotFound(op.shape_id));
        }
    }
    Ok(())
}
