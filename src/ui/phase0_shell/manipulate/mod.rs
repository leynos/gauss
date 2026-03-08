//! Manipulate-mode event adapter for `SelectTool`.
//!
//! Pointer input is translated to model-layer `ToolInputEvent` values. The
//! `SelectTool` FSM emits commands, and `Phase0Shell` applies those commands.

#![expect(
    clippy::float_arithmetic,
    reason = "manipulate mode uses floating-point tolerances and deltas"
)]

use gpui::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels};

use crate::model::{
    HitTestIndex, SelectDragDocumentSnapshot, SelectPointerDownInput, SelectPointerMoveInput,
    SelectPointerUpInput, SelectTool, SelectToolState, Tool, ToolInputEvent, Vec2,
};

use super::{HoverCache, Phase0Shell, draw::ToolMode};

/// Default hit-test tolerance in screen pixels.
const HIT_TOLERANCE_PX: f32 = 4.0;

impl Phase0Shell {
    pub(super) fn handle_canvas_mouse_down(&mut self, event: &MouseDownEvent) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.button != MouseButton::Left {
            return false;
        }

        let cursor_world = cursor_world(&self.state.viewport, event.position);
        let tolerance_world = HIT_TOLERANCE_PX / self.state.viewport.zoom();
        let hit = self
            .hit_test_index()
            .pointer_hit(cursor_world, tolerance_world);

        let transition = Tool::transition(
            &SelectTool,
            self.state.tool_mode,
            self.state.edge_mode,
            ToolInputEvent::SelectPointerDown {
                input: Box::new(SelectPointerDownInput {
                    drag_snapshot: SelectDragDocumentSnapshot::from_document(&self.state.document),
                    previous_selection: self.state.selection.clone(),
                    hit,
                    cursor_world,
                    is_shift_held: event.modifiers.shift,
                }),
            },
        );

        self.apply_tool_commands(transition.commands)
    }

    pub(super) fn handle_canvas_mouse_move(&mut self, event: &MouseMoveEvent) -> bool {
        let did_update_hover = event.pressed_button != Some(MouseButton::Left)
            && self.update_hover_hit(event.position);
        let is_dragging = matches!(self.select_tool_state, SelectToolState::Dragging(_));
        let did_apply_pointer_move = self.handle_pointer_event(
            event.position,
            event.pressed_button == Some(MouseButton::Left),
            |cursor_world| ToolInputEvent::SelectPointerMove {
                input: Box::new(SelectPointerMoveInput {
                    is_dragging,
                    cursor_world,
                    has_primary_button: true,
                }),
            },
        );

        did_update_hover || did_apply_pointer_move
    }

    pub(super) fn handle_canvas_mouse_up(&mut self, event: &MouseUpEvent) -> bool {
        let select_tool_state = self.select_tool_state.clone();
        self.handle_pointer_event(
            event.position,
            event.button == MouseButton::Left,
            move |cursor_world| ToolInputEvent::SelectPointerUp {
                input: Box::new(SelectPointerUpInput {
                    state: select_tool_state,
                    cursor_world,
                    is_primary_button: true,
                }),
            },
        )
    }

    const fn hit_test_index(&self) -> HitTestIndex<'_> {
        HitTestIndex::from_document(&self.state.document)
    }

    fn update_hover_hit(&mut self, position: gpui::Point<Pixels>) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        let cursor_world = cursor_world(&self.state.viewport, position);
        let tolerance_world = HIT_TOLERANCE_PX / self.state.viewport.zoom();
        if let Some(ref cache) = self.hover_cache
            && cache.matches(cursor_world, tolerance_world, self.document_generation)
        {
            let next = cache.result;
            return if next == self.hover_hit {
                false
            } else {
                self.hover_hit = next;
                true
            };
        }

        let next_hover_hit = self
            .hit_test_index()
            .hover_hit(cursor_world, tolerance_world);

        self.hover_cache = Some(HoverCache {
            cursor_world,
            tolerance_world,
            generation: self.document_generation,
            result: next_hover_hit,
        });

        if next_hover_hit == self.hover_hit {
            return false;
        }

        self.hover_hit = next_hover_hit;
        true
    }

    fn handle_pointer_event<F>(
        &mut self,
        position: gpui::Point<Pixels>,
        button_valid: bool,
        create_event: F,
    ) -> bool
    where
        F: FnOnce(Vec2) -> ToolInputEvent,
    {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if !button_valid {
            return false;
        }

        let cursor_world = cursor_world(&self.state.viewport, position);
        let transition = Tool::transition(
            &SelectTool,
            self.state.tool_mode,
            self.state.edge_mode,
            create_event(cursor_world),
        );

        self.apply_tool_commands(transition.commands)
    }
}

fn cursor_world(viewport: &crate::model::Viewport, position: gpui::Point<Pixels>) -> Vec2 {
    let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
    viewport.screen_to_world(cursor_screen)
}
