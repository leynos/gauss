//! GPUI integration tests for undo/redo and history management.
//!
//! This consolidated test target groups Phase 0 history behaviour tests including:
//! - Anchor edit undo/redo
//! - Path close undo/redo
//! - Command grouping in history
//! - Drag operations undo/redo (anchors, handles, shapes)
//! - Drawing undo/redo
//! - Multi-shape drag undo/redo
//! - Reorder undo/redo
//! - Selection history
//! - History reset on open

mod common;

mod gpui_history {
    mod anchor_edit_undo;
    mod close_path_undo;
    mod command_grouping_undo;
    mod drag_anchor_undo;
    mod drag_handle_undo;
    mod drag_shape_undo;
    mod draw_undo;
    mod multi_shape_drag_undo;
    mod open_history_reset;
    mod reorder_undo;
    mod selection_history;
}
