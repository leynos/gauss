//! Manipulate-mode behaviour for the Phase 0 shell.
//!
//! Phase 0 starts with just enough manipulation support to validate the
//! end-to-end wiring:
//!
//! - click to select a shape,
//! - drag to move the selected shape, and
//! - undo restores the prior position.

use gpui::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels};

use crate::model::{Anchor, DocChange, DocOp, Document, SelItem, Selection, Shape, ShapeId, Vec2};

use super::{
    Phase0Shell,
    draw::{DocHistoryItem, ToolMode},
};

#[derive(Clone, Debug)]
pub(super) struct DragState {
    kind: DragKind,
}

#[derive(Clone, Debug)]
enum DragKind {
    MoveShape(ShapeDragState),
}

#[derive(Clone, Debug)]
struct ShapeDragState {
    shape: ShapeId,
    index: usize,
    start_cursor_world: Vec2,
    original_anchors: Vec<Anchor>,
}

impl Phase0Shell {
    pub(super) fn handle_canvas_mouse_down(&mut self, event: &MouseDownEvent) -> bool {
        if self.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.button != MouseButton::Left {
            return false;
        }

        let cursor_world = cursor_world(&self.viewport, event.position);
        let tolerance_world = 4.0 / self.viewport.zoom;
        let hit = hit_test_topmost_shape(&self.document, cursor_world, tolerance_world);

        let previous_selection = self.selection.clone();
        self.selection = match hit {
            Some((_, shape_id)) => Selection {
                items: vec![SelItem::Shape(shape_id)],
            },
            None => Selection::empty(),
        };

        self.drag_state = hit.and_then(|(index, shape_id)| {
            start_shape_drag(&self.document, index, shape_id, cursor_world).map(|drag| DragState {
                kind: DragKind::MoveShape(drag),
            })
        });

        self.selection != previous_selection || self.drag_state.is_some()
    }

    pub(super) fn handle_canvas_mouse_move(&mut self, event: &MouseMoveEvent) -> bool {
        if self.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.pressed_button != Some(MouseButton::Left) {
            return false;
        }

        let Some(drag_state) = &self.drag_state else {
            return false;
        };

        let cursor_world = cursor_world(&self.viewport, event.position);
        match &drag_state.kind {
            DragKind::MoveShape(shape_drag) => {
                apply_shape_drag_preview(&mut self.document, shape_drag, cursor_world)
            }
        }
    }

    pub(super) fn handle_canvas_mouse_up(&mut self, event: &MouseUpEvent) -> bool {
        if self.tool_mode != ToolMode::Manipulate {
            return false;
        }

        if event.button != MouseButton::Left {
            return false;
        }

        let Some(drag_state) = self.drag_state.take() else {
            return false;
        };

        let cursor_world = cursor_world(&self.viewport, event.position);
        match drag_state.kind {
            DragKind::MoveShape(shape_drag) => self.finish_shape_drag(&shape_drag, cursor_world),
        }
    }

    fn finish_shape_drag(&mut self, drag: &ShapeDragState, cursor_world: Vec2) -> bool {
        let delta = cursor_world.sub(drag.start_cursor_world);

        if delta.x.abs() <= f32::EPSILON && delta.y.abs() <= f32::EPSILON {
            // Restore the original geometry to avoid accumulating tiny deltas.
            let _did_restore = apply_shape_drag_to_doc(&mut self.document, drag, Vec2::ZERO);
            return false;
        }

        let did_update = apply_shape_drag_to_doc(&mut self.document, drag, delta);
        if !did_update {
            return false;
        }

        let change = DocChange {
            ops: vec![DocOp::MoveShape {
                shape: drag.shape,
                delta,
            }],
        };
        self.document_history.push(DocHistoryItem::new(change));
        true
    }
}

fn cursor_world(viewport: &crate::model::Viewport, position: gpui::Point<Pixels>) -> Vec2 {
    let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
    viewport.screen_to_world(cursor_screen)
}

fn start_shape_drag(
    doc: &Document,
    index: usize,
    shape_id: ShapeId,
    cursor_world: Vec2,
) -> Option<ShapeDragState> {
    let shape = doc.shapes.get(index)?;
    if shape.id != shape_id {
        return None;
    }

    Some(ShapeDragState {
        shape: shape_id,
        index,
        start_cursor_world: cursor_world,
        original_anchors: shape.path.anchors.clone(),
    })
}

fn apply_shape_drag_preview(doc: &mut Document, drag: &ShapeDragState, cursor_world: Vec2) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_shape_drag_to_doc(doc, drag, delta)
}

fn apply_shape_drag_to_doc(doc: &mut Document, drag: &ShapeDragState, delta: Vec2) -> bool {
    let Some(shape) = doc.shapes.get_mut(drag.index) else {
        return false;
    };
    if shape.id != drag.shape {
        return false;
    }

    restore_shape_anchors(shape, &drag.original_anchors, delta)
}

fn restore_shape_anchors(shape: &mut Shape, original: &[Anchor], delta: Vec2) -> bool {
    if shape.path.anchors.len() != original.len() {
        return false;
    }

    for (current, start) in shape.path.anchors.iter_mut().zip(original.iter()) {
        current.pos = start.pos.add(delta);
        current.handle_in = start.handle_in.map(|p| p.add(delta));
        current.handle_out = start.handle_out.map(|p| p.add(delta));
    }

    true
}

fn hit_test_topmost_shape(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<(usize, ShapeId)> {
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, shape)| {
            hit_test_shape_bbox(shape, cursor_world, tolerance_world).then_some((index, shape.id))
        })
}

fn hit_test_shape_bbox(shape: &Shape, cursor_world: Vec2, tolerance_world: f32) -> bool {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for anchor in &shape.path.anchors {
        min_x = min_x.min(anchor.pos.x);
        min_y = min_y.min(anchor.pos.y);
        max_x = max_x.max(anchor.pos.x);
        max_y = max_y.max(anchor.pos.y);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return false;
    }

    cursor_world.x >= (min_x - tolerance_world)
        && cursor_world.x <= (max_x + tolerance_world)
        && cursor_world.y >= (min_y - tolerance_world)
        && cursor_world.y <= (max_y + tolerance_world)
}
