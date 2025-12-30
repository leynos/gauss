//! Manipulate-mode behaviour for the Phase 0 shell.
//!
//! Phase 0 starts with just enough manipulation support to validate the
//! end-to-end wiring:
//!
//! - click to select a shape,
//! - drag to move the selected shape, and
//! - undo restores the prior position.

#![expect(
    clippy::float_arithmetic,
    reason = "manipulate mode uses floating-point tolerances and deltas"
)]

use gpui::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels};

use crate::model::Vec2;

use super::{
    Phase0Shell,
    draw::{DocHistoryItem, ToolMode},
};

mod drag;
mod handle_drag;
mod hit_test;
mod selection;

pub(super) use drag::DragState;

use drag::{DragStartInput, apply_drag_preview, finish_drag, start_drag};
use hit_test::hit_under_cursor;
use selection::{can_drag_shape_bbox, selection_for_hit};

impl Phase0Shell {
    pub(super) fn handle_canvas_mouse_down(&mut self, event: &MouseDownEvent) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.button != MouseButton::Left {
            return false;
        }

        let cursor_world = cursor_world(&self.state.viewport, event.position);
        let tolerance_world = 4.0 / self.state.viewport.zoom();
        let hit = hit_under_cursor(&self.state.document, cursor_world, tolerance_world);

        let previous_selection = self.state.selection.clone();
        let can_drag_shape_bbox = can_drag_shape_bbox(&previous_selection, hit);
        let new_selection = selection_for_hit(&previous_selection, hit, event.modifiers.shift);
        let did_change_selection = new_selection != previous_selection;
        if did_change_selection {
            self.record_selection_change(previous_selection, new_selection.clone());
        }
        self.state.selection = new_selection;

        self.drag_state = if event.modifiers.shift {
            None
        } else {
            start_drag(
                &DragStartInput::new(
                    &self.state.document,
                    &self.state.selection,
                    cursor_world,
                    can_drag_shape_bbox,
                ),
                hit,
            )
        };

        did_change_selection || self.drag_state.is_some()
    }

    pub(super) fn handle_canvas_mouse_move(&mut self, event: &MouseMoveEvent) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.pressed_button != Some(MouseButton::Left) {
            return false;
        }

        let Some(drag_state) = &self.drag_state else {
            return false;
        };

        let cursor_world = cursor_world(&self.state.viewport, event.position);
        apply_drag_preview(&mut self.state.document, drag_state, cursor_world)
    }

    pub(super) fn handle_canvas_mouse_up(&mut self, event: &MouseUpEvent) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.button != MouseButton::Left {
            return false;
        }

        let Some(drag_state) = self.drag_state.take() else {
            return false;
        };

        let cursor_world = cursor_world(&self.state.viewport, event.position);
        finish_drag(&mut self.state.document, drag_state, cursor_world)
            .map(|change| {
                self.document_history.push(DocHistoryItem::new(change));
            })
            .is_some()
    }
}

fn cursor_world(viewport: &crate::model::Viewport, position: gpui::Point<Pixels>) -> Vec2 {
    let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
    viewport.screen_to_world(cursor_screen)
}
