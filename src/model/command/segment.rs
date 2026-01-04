//! Segment toggle command helpers.

use crate::model::{Document, EngineState, SegmentKind, SelItem, Shape, Vec2};

use super::Command;
use super::error::UserError;
use super::inverse::CommandInverse;
use super::types::SegmentChange;

pub(super) fn prepare_toggle_segment_kind(state: &EngineState) -> Result<Command, UserError> {
    if state.selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    // Find selected segments
    let mut changes = Vec::new();

    for item in &state.selection.items {
        let SelItem::Segment { shape, seg } = item else {
            continue;
        };

        let Some(shape_index) = state.document.find_index(*shape) else {
            return Err(UserError::ShapeNotFound(*shape));
        };

        let Some(shape_data) = state.document.shapes.get(shape_index) else {
            return Err(UserError::ShapeNotFound(*shape));
        };

        let Some(&old_kind) = shape_data.path.segments.get(*seg) else {
            return Err(UserError::SegmentNotFound(*shape, *seg));
        };

        let start_anchor = shape_data.path.anchors.get(*seg);
        let end_anchor = shape_data.path.anchors.get(*seg + 1);

        let (new_kind, new_start_handle_out, new_end_handle_in) = match old_kind {
            SegmentKind::Line => {
                // Convert Line to Cubic: synthesise linear handles
                let (h_out, h_in) = if let (Some(start), Some(end)) = (start_anchor, end_anchor) {
                    // Simple handle synthesis: 1/3 of the way along the segment
                    let dx = end.pos.x - start.pos.x;
                    let dy = end.pos.y - start.pos.y;
                    let h_out = Vec2::new(start.pos.x + dx / 3.0, start.pos.y + dy / 3.0);
                    let h_in = Vec2::new(end.pos.x - dx / 3.0, end.pos.y - dy / 3.0);
                    (Some(h_out), Some(h_in))
                } else {
                    (None, None)
                };
                (SegmentKind::Cubic, h_out, h_in)
            }
            SegmentKind::Cubic => {
                // Convert Cubic to Line: clear handles
                (SegmentKind::Line, None, None)
            }
        };

        let old_start_handle_out = start_anchor.and_then(|a| a.handle_out);
        let old_end_handle_in = end_anchor.and_then(|a| a.handle_in);

        changes.push(SegmentChange {
            shape_id: *shape,
            segment_index: *seg,
            old_kind,
            new_kind,
            old_start_handle_out,
            new_start_handle_out,
            old_end_handle_in,
            new_end_handle_in,
        });
    }

    if changes.is_empty() {
        return Err(UserError::InvalidOperation("No segments selected"));
    }

    Ok(Command::SetSegmentKind { changes })
}

pub(super) fn apply_set_segment_kind(
    doc: &mut Document,
    changes: &[SegmentChange],
    command_name: &'static str,
) -> CommandInverse {
    for change in changes {
        if let Some(shape) = doc.get_mut(change.shape_id) {
            apply_segment_change(shape, change);
        }
    }

    // Create inverse with swapped old/new
    let inverse_changes = changes
        .iter()
        .map(|c| SegmentChange {
            shape_id: c.shape_id,
            segment_index: c.segment_index,
            old_kind: c.new_kind,
            new_kind: c.old_kind,
            old_start_handle_out: c.new_start_handle_out,
            new_start_handle_out: c.old_start_handle_out,
            old_end_handle_in: c.new_end_handle_in,
            new_end_handle_in: c.old_end_handle_in,
        })
        .collect();

    CommandInverse::RestoreSegmentKinds {
        command_name,
        changes: inverse_changes,
    }
}

pub(super) fn apply_restore_segment_kinds(doc: &mut Document, changes: &[SegmentChange]) {
    for change in changes {
        if let Some(shape) = doc.get_mut(change.shape_id) {
            apply_segment_change(shape, change);
        }
    }
}

fn apply_segment_change(shape: &mut Shape, change: &SegmentChange) {
    if let Some(seg) = shape.path.segments.get_mut(change.segment_index) {
        *seg = change.new_kind;
    }
    if let Some(anchor) = shape.path.anchors.get_mut(change.segment_index) {
        anchor.handle_out = change.new_start_handle_out;
    }
    let end_index = change.segment_index + 1;
    if let Some(anchor) = shape.path.anchors.get_mut(end_index) {
        anchor.handle_in = change.new_end_handle_in;
    }
}
