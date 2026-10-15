//! Anchor insertion and deletion for Phase 0.
//!
//! Phase 0 keeps anchor edits minimal while routing the mutations through the
//! Command pipeline so undo/redo stays consistent.

use crate::model::{Action, SelItem, SelectToolState, Selection, ShapeId, prepare_command};

use super::{Phase0Shell, draw::ToolMode};

impl Phase0Shell {
    /// Insert an anchor on the first selected segment in manipulate mode.
    ///
    /// Returns `true` when the command applies and the new anchor is selected.
    /// For example, inserting after segment index `n` selects anchor `n + 1`.
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

    /// Delete selected anchors in manipulate mode and clear the selection.
    ///
    /// Returns `true` when the command applies successfully.
    /// For example, a successful deletion leaves the selection empty.
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

/// Find the first selected segment, returning its shape and segment index.
///
/// For example, non-segment items before the first segment are skipped.
fn first_selected_segment(items: &[SelItem]) -> Option<(ShapeId, usize)> {
    items.iter().find_map(|item| match item {
        SelItem::Segment { shape, seg } => Some((*shape, *seg)),
        _ => None,
    })
}
