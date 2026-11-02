//! Anchor insertion and deletion for Phase 0.
//!
//! Phase 0 keeps anchor edits minimal while routing the mutations through the
//! Command pipeline so undo/redo stays consistent.

use super::{Phase0Shell, draw::ToolMode};
use crate::model::{Action, SelItem, SelectToolState, Selection, ShapeId, prepare_command};

impl Phase0Shell {
    /// Insert an anchor into the first selected segment.
    ///
    /// Only valid in manipulate mode. Returns `false` when no segment is
    /// selected or the command cannot be prepared or applied.
    pub(super) fn insert_anchor_on_selected_segment(&mut self) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let Some((shape_id, seg_index)) = first_selected_segment(&self.state.selection.items)
        else {
            return false;
        };

        let command = match prepare_command(Action::InsertAnchorOnSegment, &self.state) {
            Ok(command) => command,
            Err(error) => {
                log::error!("prepare insert anchor command failed: {error}");
                return false;
            }
        };

        if let Err(error) = self.apply_command(command) {
            log::error!("apply insert anchor command failed: {error}");
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
        self.select_tool_state = SelectToolState::Idle;
        true
    }

    /// Delete the selected anchors through the command pipeline.
    ///
    /// Only valid in manipulate mode. Returns `false` when the command cannot
    /// be prepared or applied.
    pub(super) fn delete_selected_anchors(&mut self) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let command = match prepare_command(Action::DeleteSelectedAnchors, &self.state) {
            Ok(command) => command,
            Err(error) => {
                log::error!("prepare delete anchors command failed: {error}");
                return false;
            }
        };

        if let Err(error) = self.apply_command(command) {
            log::error!("apply delete anchors command failed: {error}");
            return false;
        }

        let previous_selection = self.state.selection.clone();
        let new_selection = Selection::empty();
        self.record_selection_change(previous_selection, new_selection.clone());
        self.state.selection = new_selection;
        self.select_tool_state = SelectToolState::Idle;
        true
    }
}

/// Return the shape and segment index of the first segment selection.
fn first_selected_segment(items: &[SelItem]) -> Option<(ShapeId, usize)> {
    items.iter().find_map(|item| match item {
        SelItem::Segment { shape, seg } => Some((*shape, *seg)),
        _ => None,
    })
}
