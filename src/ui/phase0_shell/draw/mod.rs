//! Draw-mode behaviour and document command dispatch for the Phase 0 shell.
//!
//! Phase 0 intentionally keeps draw mode simple:
//!
//! - Click appends anchors to an open path.
//! - Clicking near the first anchor closes the path.
//! - Each click is one undo step.
//! - When in "Bezier (auto)" mode, we synthesise cubic handles using a
//!   Catmull–Rom-to-cubic conversion.

#![expect(
    clippy::float_arithmetic,
    reason = "draw mode operates on floating-point geometry and handles"
)]

use crate::model::{
    Anchor, Command, EdgeMode, PaintStyle, PathGeom, SegmentKind, Shape, ShapeId, ShapeInsertion,
    UserError, Vec2,
};

use super::Phase0Shell;

mod handles;

use self::handles::{clear_line_segment_handles, close_shape, update_catmull_rom_handles};

const SNAP_RADIUS_PX: f32 = 10.0;

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
        let Some(active) = self.state.active_path else {
            return self.start_new_open_shape(cursor_world);
        };

        let Some(index) = self.state.document.find_index(active) else {
            self.state.active_path = None;
            return self.start_new_open_shape(cursor_world);
        };

        let Some(existing) = self.state.document.shape_at(index).cloned() else {
            self.state.active_path = None;
            return false;
        };

        if should_close_path(&existing.path, cursor_world, self.state.viewport.zoom()) {
            let closed = close_shape(
                existing.clone(),
                segment_kind_for_edge_mode(self.state.edge_mode),
            );
            let command = Command::ClosePath {
                replacement: crate::model::ShapeReplacement {
                    shape_index: index,
                    old_shape: existing,
                    new_shape: closed,
                },
            };
            if self.apply_command(command).is_err() {
                return false;
            }
            self.state.tool_mode = ToolMode::Manipulate;
            self.state.active_path = None;
            return true;
        }

        let appended = append_anchor(existing.clone(), cursor_world, self.state.edge_mode);
        let command = Command::InsertAnchor {
            replacement: crate::model::ShapeReplacement {
                shape_index: index,
                old_shape: existing,
                new_shape: appended,
            },
        };
        self.apply_command(command).is_ok()
    }

    fn start_new_open_shape(&mut self, cursor_world: Vec2) -> bool {
        let shape_id = self.state.document.allocate_shape_id();
        let shape = new_open_shape(shape_id, cursor_world, self.state.current_style.clone());
        let index = self.state.document.len();

        let command = Command::InsertShape {
            insertion: ShapeInsertion { index, shape },
        };

        if self.apply_command(command).is_err() {
            return false;
        }

        self.state.active_path = Some(shape_id);
        true
    }

    pub(super) fn apply_command(&mut self, command: Command) -> Result<(), UserError> {
        let inverse = command.apply(&mut self.state.document)?;
        self.document_history.record(command, inverse);
        self.last_history_error = None;
        Ok(())
    }

    pub(super) fn undo_document(&mut self) {
        match self.document_history.undo(&mut self.state.document) {
            Ok(()) => self.last_history_error = None,
            Err(error) => {
                log::error!("{error}");
                self.last_history_error = Some(error.to_string());
            }
        }
    }

    pub(super) fn redo_document(&mut self) {
        match self.document_history.redo(&mut self.state.document) {
            Ok(()) => self.last_history_error = None,
            Err(error) => {
                log::error!("{error}");
                self.last_history_error = Some(error.to_string());
            }
        }
    }
}

fn new_open_shape(id: ShapeId, first_anchor: Vec2, style: PaintStyle) -> Shape {
    Shape {
        id,
        z: 0,
        style,
        path: PathGeom {
            anchors: vec![Anchor::new(first_anchor)],
            segments: Vec::new(),
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

const fn segment_kind_for_edge_mode(edge_mode: DrawEdgeMode) -> SegmentKind {
    match edge_mode {
        DrawEdgeMode::Line => SegmentKind::Line,
        DrawEdgeMode::BezierAuto => SegmentKind::Cubic,
    }
}

fn append_anchor(mut shape: Shape, pos: Vec2, edge_mode: DrawEdgeMode) -> Shape {
    let new_anchor_index = shape.path.anchors.len();
    shape.path.anchors.push(Anchor::new(pos));
    shape
        .path
        .segments
        .push(segment_kind_for_edge_mode(edge_mode));

    match edge_mode {
        DrawEdgeMode::Line => {
            clear_line_segment_handles(&mut shape.path, new_anchor_index);
        }
        DrawEdgeMode::BezierAuto => {
            update_catmull_rom_handles(&mut shape.path, new_anchor_index);
        }
    }

    shape
}

fn should_close_path(path: &PathGeom, cursor_world: Vec2, zoom: f32) -> bool {
    let Some(first) = path.anchors.first() else {
        return false;
    };

    if path.anchors.len() < 3 {
        return false;
    }

    let snap_world = SNAP_RADIUS_PX / zoom;
    first.pos.distance(cursor_world) <= snap_world
}
