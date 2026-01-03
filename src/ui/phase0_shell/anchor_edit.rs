//! Anchor insertion and deletion for Phase 0.
//!
//! Phase 0 keeps anchor edits minimal while routing the mutations through the
//! Command pipeline so undo/redo stays consistent.

use crate::model::{Action, SelItem, Selection, prepare_command};

use super::{Phase0Shell, draw::ToolMode};

impl Phase0Shell {
    pub(super) fn insert_anchor_on_selected_segment(&mut self) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let Some((shape_id, seg_index)) = first_selected_segment(&self.state.selection.items)
        else {
            return false;
        };

        let Ok(command) = prepare_command(Action::InsertAnchorOnSegment, &self.state) else {
            return false;
        };

        if self.apply_command(command).is_err() {
            return false;
        }

        let previous_selection = self.state.selection.clone();
        let new_selection = Selection {
            items: vec![SelItem::Anchor {
                shape: shape_id,
                anchor: seg_index + 1,
            }],
        };
        self.record_selection_change(previous_selection, new_selection.clone());
        self.state.selection = new_selection;
        self.drag_state = None;
        true
    }

    pub(super) fn delete_selected_anchors(&mut self) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let Ok(command) = prepare_command(Action::DeleteSelectedAnchors, &self.state) else {
            return false;
        };

        if self.apply_command(command).is_err() {
            return false;
        }

        let previous_selection = self.state.selection.clone();
        let new_selection = Selection::empty();
        self.record_selection_change(previous_selection, new_selection.clone());
        self.state.selection = new_selection;
        self.drag_state = None;
        true
    }
}

fn first_selected_segment(items: &[SelItem]) -> Option<(crate::model::ShapeId, usize)> {
    items.iter().find_map(|item| match item {
        SelItem::Segment { shape, seg } => Some((*shape, *seg)),
        _ => None,
    })
}
