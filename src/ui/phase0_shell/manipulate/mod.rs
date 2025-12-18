//! Manipulate-mode behaviour for the Phase 0 shell.
//!
//! This module is split into submodules to stay under the repository’s
//! per-file line limit.
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

mod handle_drag;
mod hit_test;

use self::handle_drag::{
    HandleDragState, apply_handle_drag_preview, finish_handle_drag, start_handle_drag,
};
use self::hit_test::{
    AnchorHit, HandleHit, HandleHitKind, SegmentHit, hit_test_topmost_anchor,
    hit_test_topmost_handle, hit_test_topmost_segment, hit_test_topmost_shape,
};

#[derive(Clone, Debug)]
pub(super) struct DragState {
    kind: DragKind,
}

#[derive(Clone, Debug)]
enum DragKind {
    Shapes(ShapesDragState),
    Anchor(AnchorDragState),
    Handle(HandleDragState),
}

#[derive(Clone, Debug)]
struct ShapesDragState {
    start_cursor_world: Vec2,
    shapes: Vec<DraggedShape>,
}

#[derive(Clone, Debug)]
struct DraggedShape {
    shape: ShapeId,
    index: usize,
    original_anchors: Vec<Anchor>,
}

#[derive(Clone, Debug)]
struct AnchorDragState {
    shape: ShapeId,
    shape_index: usize,
    anchor_index: usize,
    start_cursor_world: Vec2,
    original_anchor: Anchor,
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
        let hit = hit_under_cursor(&self.document, cursor_world, tolerance_world);

        let previous_selection = self.selection.clone();
        let hit_selection = selection_for_hit(hit);
        let new_selection = if event.modifiers.shift {
            toggle_selection_item(previous_selection.clone(), &hit_selection)
        } else if hit_selection
            .items
            .first()
            .is_some_and(|item| previous_selection.contains(item))
        {
            // Clicking an already-selected item should not collapse the current
            // selection. This is critical for multi-select move gestures.
            previous_selection.clone()
        } else {
            hit_selection
        };
        let did_change_selection = new_selection != previous_selection;
        if did_change_selection {
            self.record_selection_change(previous_selection, new_selection.clone());
        }
        self.selection = new_selection;

        self.drag_state = if event.modifiers.shift {
            None
        } else {
            drag_state_for_hit(&self.document, hit, cursor_world, &self.selection)
        };

        did_change_selection || self.drag_state.is_some()
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
            DragKind::Shapes(shape_drag) => {
                apply_shapes_drag_preview(&mut self.document, shape_drag, cursor_world)
            }
            DragKind::Anchor(anchor_drag) => {
                apply_anchor_drag_preview(&mut self.document, anchor_drag, cursor_world)
            }
            DragKind::Handle(handle_drag) => {
                apply_handle_drag_preview(&mut self.document, handle_drag, cursor_world)
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
            DragKind::Shapes(shape_drag) => self.finish_shapes_drag(&shape_drag, cursor_world),
            DragKind::Anchor(anchor_drag) => self.finish_anchor_drag(&anchor_drag, cursor_world),
            DragKind::Handle(handle_drag) => {
                finish_handle_drag(&mut self.document, &handle_drag, cursor_world).is_some_and(
                    |op| {
                        self.document_history
                            .push(DocHistoryItem::new(DocChange { ops: vec![op] }));
                        true
                    },
                )
            }
        }
    }

    fn finish_shapes_drag(&mut self, drag: &ShapesDragState, cursor_world: Vec2) -> bool {
        let delta = cursor_world.sub(drag.start_cursor_world);

        if delta.x.abs() <= f32::EPSILON && delta.y.abs() <= f32::EPSILON {
            // Restore the original geometry to avoid accumulating tiny deltas.
            let _did_restore = apply_shapes_drag_to_doc(&mut self.document, drag, Vec2::ZERO);
            return false;
        }

        let did_update = apply_shapes_drag_to_doc(&mut self.document, drag, delta);
        if !did_update {
            return false;
        }

        let ops = drag
            .shapes
            .iter()
            .map(|dragged| DocOp::MoveShape {
                shape: dragged.shape,
                delta,
            })
            .collect();

        let change = DocChange { ops };
        self.document_history.push(DocHistoryItem::new(change));
        true
    }

    fn finish_anchor_drag(&mut self, drag: &AnchorDragState, cursor_world: Vec2) -> bool {
        let delta = cursor_world.sub(drag.start_cursor_world);

        if delta.x.abs() <= f32::EPSILON && delta.y.abs() <= f32::EPSILON {
            let _did_restore = apply_anchor_drag_to_doc(&mut self.document, drag, Vec2::ZERO);
            return false;
        }

        let did_update = apply_anchor_drag_to_doc(&mut self.document, drag, delta);
        if !did_update {
            return false;
        }

        let mut ops = vec![DocOp::MoveAnchor {
            shape: drag.shape,
            anchor: drag.anchor_index,
            from: drag.original_anchor.pos,
            to: drag.original_anchor.pos.add(delta),
        }];

        if drag.original_anchor.handle_in.is_some() {
            ops.push(DocOp::MoveHandleIn {
                shape: drag.shape,
                anchor: drag.anchor_index,
                from: drag.original_anchor.handle_in,
                to: drag.original_anchor.handle_in.map(|p| p.add(delta)),
            });
        }

        if drag.original_anchor.handle_out.is_some() {
            ops.push(DocOp::MoveHandleOut {
                shape: drag.shape,
                anchor: drag.anchor_index,
                from: drag.original_anchor.handle_out,
                to: drag.original_anchor.handle_out.map(|p| p.add(delta)),
            });
        }

        self.document_history
            .push(DocHistoryItem::new(DocChange { ops }));
        true
    }
}

#[derive(Clone, Copy, Debug)]
enum MouseDownHit {
    Handle(HandleHit),
    Anchor(AnchorHit),
    Segment(SegmentHit),
    Shape { id: ShapeId },
    None,
}

fn hit_under_cursor(doc: &Document, cursor_world: Vec2, tolerance_world: f32) -> MouseDownHit {
    if let Some(hit) = hit_test_topmost_handle(doc, cursor_world, tolerance_world) {
        return MouseDownHit::Handle(hit);
    }

    if let Some(hit) = hit_test_topmost_anchor(doc, cursor_world, tolerance_world) {
        return MouseDownHit::Anchor(hit);
    }

    if let Some(hit) = hit_test_topmost_segment(doc, cursor_world, tolerance_world) {
        return MouseDownHit::Segment(hit);
    }

    hit_test_topmost_shape(doc, cursor_world, tolerance_world)
        .map_or(MouseDownHit::None, |id| MouseDownHit::Shape { id })
}

fn selection_for_hit(hit: MouseDownHit) -> Selection {
    match hit {
        MouseDownHit::Handle(handle_hit) => Selection {
            items: vec![match handle_hit.kind {
                HandleHitKind::In => SelItem::HandleIn {
                    shape: handle_hit.shape_id,
                    anchor: handle_hit.anchor_index,
                },
                HandleHitKind::Out => SelItem::HandleOut {
                    shape: handle_hit.shape_id,
                    anchor: handle_hit.anchor_index,
                },
            }],
        },
        MouseDownHit::Anchor(anchor_hit) => Selection {
            items: vec![SelItem::Anchor {
                shape: anchor_hit.shape_id,
                anchor: anchor_hit.anchor_index,
            }],
        },
        MouseDownHit::Segment(segment_hit) => Selection {
            items: vec![SelItem::Segment {
                shape: segment_hit.shape_id,
                seg: segment_hit.seg_index,
            }],
        },
        MouseDownHit::Shape { id } => Selection {
            items: vec![SelItem::Shape(id)],
        },
        MouseDownHit::None => Selection::empty(),
    }
}

fn toggle_selection_item(current: Selection, hit_selection: &Selection) -> Selection {
    let Some(item) = hit_selection.items.first().cloned() else {
        return current;
    };

    let mut items = current.items;
    if let Some(pos) = items.iter().position(|existing| existing == &item) {
        items.remove(pos);
    } else {
        items.push(item);
    }

    Selection { items }
}

fn selection_shape_ids(selection: &Selection) -> Vec<ShapeId> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut ids = Vec::new();
    for item in &selection.items {
        let SelItem::Shape(id) = item else {
            continue;
        };
        if seen.insert(*id) {
            ids.push(*id);
        }
    }

    ids
}

fn drag_state_for_hit(
    doc: &Document,
    hit: MouseDownHit,
    cursor_world: Vec2,
    selection: &Selection,
) -> Option<DragState> {
    match hit {
        MouseDownHit::Handle(handle_hit) => {
            start_handle_drag(doc, handle_hit, cursor_world).map(|drag| DragState {
                kind: DragKind::Handle(drag),
            })
        }
        MouseDownHit::Anchor(anchor_hit) => {
            start_anchor_drag(doc, anchor_hit, cursor_world).map(|drag| DragState {
                kind: DragKind::Anchor(drag),
            })
        }
        MouseDownHit::Segment(segment_hit) => start_shapes_drag(
            doc,
            selection,
            cursor_world,
            DraggedShapeHit::new(segment_hit.shape_id),
        )
        .map(|drag| DragState {
            kind: DragKind::Shapes(drag),
        }),
        MouseDownHit::Shape { id } => {
            start_shapes_drag(doc, selection, cursor_world, DraggedShapeHit::new(id)).map(|drag| {
                DragState {
                    kind: DragKind::Shapes(drag),
                }
            })
        }
        MouseDownHit::None => None,
    }
}

fn cursor_world(viewport: &crate::model::Viewport, position: gpui::Point<Pixels>) -> Vec2 {
    let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
    viewport.screen_to_world(cursor_screen)
}

#[derive(Clone, Copy, Debug)]
struct DraggedShapeHit {
    shape: ShapeId,
}

impl DraggedShapeHit {
    const fn new(shape: ShapeId) -> Self {
        Self { shape }
    }
}

fn start_shapes_drag(
    doc: &Document,
    selection: &Selection,
    cursor_world: Vec2,
    hit: DraggedShapeHit,
) -> Option<ShapesDragState> {
    let drag_all_selected = selection.contains(&SelItem::Shape(hit.shape));
    let shape_ids = if drag_all_selected {
        selection_shape_ids(selection)
    } else {
        vec![hit.shape]
    };

    let mut shapes = Vec::new();
    for shape_id in shape_ids {
        let Some(index) = doc.find_index(shape_id) else {
            continue;
        };
        let shape = doc.shapes.get(index)?;
        shapes.push(DraggedShape {
            shape: shape_id,
            index,
            original_anchors: shape.path.anchors.clone(),
        });
    }

    (!shapes.is_empty()).then_some(ShapesDragState {
        start_cursor_world: cursor_world,
        shapes,
    })
}

fn start_anchor_drag(
    doc: &Document,
    hit: AnchorHit,
    cursor_world: Vec2,
) -> Option<AnchorDragState> {
    let shape = doc.shapes.get(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }
    let anchor = shape.path.anchors.get(hit.anchor_index)?.clone();

    Some(AnchorDragState {
        shape: hit.shape_id,
        shape_index: hit.shape_index,
        anchor_index: hit.anchor_index,
        start_cursor_world: cursor_world,
        original_anchor: anchor,
    })
}

fn apply_shapes_drag_preview(
    doc: &mut Document,
    drag: &ShapesDragState,
    cursor_world: Vec2,
) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_shapes_drag_to_doc(doc, drag, delta)
}

fn apply_shapes_drag_to_doc(doc: &mut Document, drag: &ShapesDragState, delta: Vec2) -> bool {
    let mut did_update_any = false;

    for dragged in &drag.shapes {
        let Some(shape) = doc.shapes.get_mut(dragged.index) else {
            continue;
        };
        if shape.id != dragged.shape {
            continue;
        }

        did_update_any |= restore_shape_anchors(shape, &dragged.original_anchors, delta);
    }

    did_update_any
}

fn apply_anchor_drag_preview(
    doc: &mut Document,
    drag: &AnchorDragState,
    cursor_world: Vec2,
) -> bool {
    let delta = cursor_world.sub(drag.start_cursor_world);
    apply_anchor_drag_to_doc(doc, drag, delta)
}

fn apply_anchor_drag_to_doc(doc: &mut Document, drag: &AnchorDragState, delta: Vec2) -> bool {
    let Some(shape) = doc.shapes.get_mut(drag.shape_index) else {
        return false;
    };
    if shape.id != drag.shape {
        return false;
    }
    let Some(anchor) = shape.path.anchors.get_mut(drag.anchor_index) else {
        return false;
    };

    anchor.pos = drag.original_anchor.pos.add(delta);
    anchor.handle_in = drag.original_anchor.handle_in.map(|p| p.add(delta));
    anchor.handle_out = drag.original_anchor.handle_out.map(|p| p.add(delta));
    true
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
