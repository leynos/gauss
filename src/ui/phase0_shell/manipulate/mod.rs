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
    SelectAnchorHit, SelectHandleHit, SelectHandleHitKind, SelectPointerDownInput,
    SelectPointerHit, SelectPointerMoveInput, SelectPointerUpInput, SelectSegmentHit,
    SelectShapeHit, SelectTool, Tool, ToolInputEvent, Vec2,
};

use super::{Phase0Shell, draw::ToolMode};

mod hit_test;

use hit_test::{
    AnchorHit, HandleHit, HandleHitKind, MouseDownHit, SegmentHit, ShapeHit, hit_under_cursor,
};

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

        let transition = Tool::transition(
            &SelectTool,
            self.state.tool_mode,
            self.state.edge_mode,
            ToolInputEvent::SelectPointerDown {
                input: Box::new(SelectPointerDownInput {
                    document: self.state.document.clone(),
                    previous_selection: self.state.selection.clone(),
                    hit: map_hit(hit),
                    cursor_world,
                    is_shift_held: event.modifiers.shift,
                }),
            },
        );

        self.apply_tool_commands(transition.commands)
    }

    pub(super) fn handle_canvas_mouse_move(&mut self, event: &MouseMoveEvent) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.pressed_button != Some(MouseButton::Left) {
            return false;
        }

        let cursor_world = cursor_world(&self.state.viewport, event.position);
        let transition = Tool::transition(
            &SelectTool,
            self.state.tool_mode,
            self.state.edge_mode,
            ToolInputEvent::SelectPointerMove {
                input: Box::new(SelectPointerMoveInput {
                    state: self.select_tool_state.clone(),
                    cursor_world,
                    has_primary_button: true,
                }),
            },
        );

        self.apply_tool_commands(transition.commands)
    }

    pub(super) fn handle_canvas_mouse_up(&mut self, event: &MouseUpEvent) -> bool {
        if self.state.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.button != MouseButton::Left {
            return false;
        }

        let cursor_world = cursor_world(&self.state.viewport, event.position);
        let transition = Tool::transition(
            &SelectTool,
            self.state.tool_mode,
            self.state.edge_mode,
            ToolInputEvent::SelectPointerUp {
                input: Box::new(SelectPointerUpInput {
                    state: self.select_tool_state.clone(),
                    cursor_world,
                    is_primary_button: true,
                }),
            },
        );

        self.apply_tool_commands(transition.commands)
    }
}

const fn map_hit(hit: MouseDownHit) -> SelectPointerHit {
    match hit {
        MouseDownHit::Handle(handle_hit) => SelectPointerHit::Handle(map_handle_hit(handle_hit)),
        MouseDownHit::Anchor(anchor_hit) => SelectPointerHit::Anchor(map_anchor_hit(anchor_hit)),
        MouseDownHit::Segment(segment_hit) => {
            SelectPointerHit::Segment(map_segment_hit(segment_hit))
        }
        MouseDownHit::Shape(shape_hit) => SelectPointerHit::Shape(map_shape_hit(shape_hit)),
        MouseDownHit::None => SelectPointerHit::None,
    }
}

const fn map_shape_hit(hit: ShapeHit) -> SelectShapeHit {
    SelectShapeHit {
        shape_index: hit.shape_index,
        shape_id: hit.shape_id,
    }
}

const fn map_anchor_hit(hit: AnchorHit) -> SelectAnchorHit {
    SelectAnchorHit {
        shape_index: hit.shape_index,
        shape_id: hit.shape_id,
        anchor_index: hit.anchor_index,
    }
}

const fn map_segment_hit(hit: SegmentHit) -> SelectSegmentHit {
    SelectSegmentHit {
        shape_index: hit.shape_index,
        shape_id: hit.shape_id,
        seg_index: hit.seg_index,
    }
}

const fn map_handle_hit(hit: HandleHit) -> SelectHandleHit {
    SelectHandleHit {
        shape_index: hit.shape_index,
        shape_id: hit.shape_id,
        anchor_index: hit.anchor_index,
        kind: map_handle_kind(hit.kind),
    }
}

const fn map_handle_kind(kind: HandleHitKind) -> SelectHandleHitKind {
    match kind {
        HandleHitKind::In => SelectHandleHitKind::In,
        HandleHitKind::Out => SelectHandleHitKind::Out,
    }
}

fn cursor_world(viewport: &crate::model::Viewport, position: gpui::Point<Pixels>) -> Vec2 {
    let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
    viewport.screen_to_world(cursor_screen)
}
