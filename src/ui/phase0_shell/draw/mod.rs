//! Draw-mode behaviour and document command dispatch for the Phase 0 shell.
//!
//! Phase 0 intentionally keeps draw mode simple:
//!
//! - Click appends anchors to an open path.
//! - Clicking near the first anchor closes the path.
//! - Each click is one undo step.
//! - When in "Bezier (auto)" mode, we synthesise cubic handles using a
//!   Catmull–Rom-to-cubic conversion.

use crate::model::{
    Command, EdgeMode, PenTool, PenToolActiveShape, PenToolClickInput, Tool, ToolCommand,
    ToolInputEvent, UserError, Vec2,
};

use super::Phase0Shell;

// Re-export tool types for backward compatibility within the UI layer.
// These are now defined in src/model/tool.rs for GPUI-independence.
pub(crate) use crate::model::ToolMode;

/// Type alias for backward compatibility during migration.
///
/// The canonical type is now `EdgeMode` in the model layer. This alias
/// preserves existing `DrawEdgeMode` references in the UI layer.
pub(super) type DrawEdgeMode = EdgeMode;

impl Phase0Shell {
    pub(super) fn handle_canvas_click(&mut self, position: gpui::Point<gpui::Pixels>) -> bool {
        let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
        self.last_canvas_click_screen = Some(cursor_screen);

        if self.state.tool_mode != ToolMode::Draw {
            return false;
        }

        let cursor_world = self.state.viewport.screen_to_world(cursor_screen);
        self.draw_click_world(cursor_world)
    }

    fn draw_click_world(&mut self, cursor_world: Vec2) -> bool {
        let Ok(active_shape) = self.snapshot_active_pen_shape() else {
            return false;
        };

        let needs_new_shape_id = self.state.active_path.is_none()
            || active_shape
                .as_ref()
                .is_none_or(|snapshot| Some(snapshot.shape.id) != self.state.active_path);
        let next_shape_id = if needs_new_shape_id {
            self.state.document.allocate_shape_id()
        } else {
            self.state
                .active_path
                .unwrap_or_else(|| self.state.document.allocate_shape_id())
        };

        let transition = Tool::transition(
            &PenTool,
            self.state.tool_mode,
            self.state.edge_mode,
            ToolInputEvent::PenCanvasClicked {
                input: Box::new(PenToolClickInput {
                    cursor_world,
                    zoom: self.state.viewport.zoom(),
                    current_style: self.state.current_style.clone(),
                    active_path: self.state.active_path,
                    active_shape,
                    next_shape_id,
                    document_len: self.state.document.len(),
                    snap_radius_px: PenToolClickInput::DEFAULT_SNAP_RADIUS_PX,
                }),
            },
        );

        self.apply_tool_commands(transition.commands)
    }

    fn snapshot_active_pen_shape(&mut self) -> Result<Option<PenToolActiveShape>, ()> {
        let Some(active_path) = self.state.active_path else {
            return Ok(None);
        };
        let Some(index) = self.state.document.find_index(active_path) else {
            return Ok(None);
        };
        let Some(existing) = self.state.document.shape_at(index).cloned() else {
            let _ = self.apply_tool_commands([ToolCommand::SetActivePath(None)]);
            return Err(());
        };
        Ok(Some(PenToolActiveShape {
            index,
            shape: existing,
        }))
    }

    pub(super) fn apply_command(&mut self, command: Command) -> Result<(), UserError> {
        self.state.apply_document_command(command)?;
        self.last_history_error = None;
        Ok(())
    }

    pub(super) fn undo_document(&mut self) {
        match self.state.undo_document() {
            Ok(()) => self.last_history_error = None,
            Err(error) => {
                log::error!("{error}");
                self.last_history_error = Some(error.to_string());
            }
        }
    }

    pub(super) fn redo_document(&mut self) {
        match self.state.redo_document() {
            Ok(()) => self.last_history_error = None,
            Err(error) => {
                log::error!("{error}");
                self.last_history_error = Some(error.to_string());
            }
        }
    }
}
