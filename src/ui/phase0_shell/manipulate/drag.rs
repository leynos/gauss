//! Dragging behaviour for Phase 0 manipulate mode.
//!
//! Drag handling is split out from the event wiring so the selection logic and
//! hit-testing remain easier to follow.

use crate::model::{
    Anchor, AnchorMovement, Command, Document, SelItem, Shape, ShapeId, ShapeMovement, Vec2,
};

use super::handle_drag::{
    HandleDragState, apply_handle_drag_preview, finish_handle_drag, start_handle_drag,
};
use super::hit_test::{AnchorHit, MouseDownHit, ShapeHit};
use super::selection::selected_shape_ids_for_drag;

#[derive(Clone, Debug)]
pub(in crate::ui::phase0_shell) struct DragState {
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

pub(super) struct DragStartInput<'a> {
    doc: &'a Document,
    selection: &'a crate::model::Selection,
    cursor_world: Vec2,
    can_drag_shape_bbox: bool,
}

impl<'a> DragStartInput<'a> {
    pub(super) const fn new(
        doc: &'a Document,
        selection: &'a crate::model::Selection,
        cursor_world: Vec2,
        can_drag_shape_bbox: bool,
    ) -> Self {
        Self {
            doc,
            selection,
            cursor_world,
            can_drag_shape_bbox,
        }
    }
}

pub(super) fn start_drag(input: &DragStartInput<'_>, hit: MouseDownHit) -> Option<DragState> {
    match hit {
        MouseDownHit::Handle(handle_hit) => {
            start_handle_drag(input.doc, handle_hit, input.cursor_world).map(|drag| DragState {
                kind: DragKind::Handle(drag),
            })
        }
        MouseDownHit::Anchor(anchor_hit) => {
            start_anchor_drag(input.doc, anchor_hit, input.cursor_world).map(|drag| DragState {
                kind: DragKind::Anchor(drag),
            })
        }
        MouseDownHit::Segment(segment_hit) => start_shapes_drag(
            input.doc,
            input.selection,
            input.cursor_world,
            ShapeHit {
                shape_index: segment_hit.shape_index,
                shape_id: segment_hit.shape_id,
            },
        )
        .map(|drag| DragState {
            kind: DragKind::Shapes(drag),
        }),
        MouseDownHit::Shape(ShapeHit {
            shape_id,
            shape_index,
        }) => {
            if !input.can_drag_shape_bbox {
                return None;
            }
            start_shapes_drag(
                input.doc,
                input.selection,
                input.cursor_world,
                ShapeHit {
                    shape_index,
                    shape_id,
                },
            )
            .map(|drag| DragState {
                kind: DragKind::Shapes(drag),
            })
        }
        MouseDownHit::None => None,
    }
}

pub(super) fn apply_drag_preview(
    doc: &mut Document,
    drag_state: &DragState,
    cursor_world: Vec2,
) -> bool {
    match &drag_state.kind {
        DragKind::Shapes(shape_drag) => apply_shapes_drag_preview(doc, shape_drag, cursor_world),
        DragKind::Anchor(anchor_drag) => apply_anchor_drag_preview(doc, anchor_drag, cursor_world),
        DragKind::Handle(handle_drag) => apply_handle_drag_preview(doc, handle_drag, cursor_world),
    }
}

pub(super) fn finish_drag(
    doc: &mut Document,
    drag_state: DragState,
    cursor_world: Vec2,
) -> Option<Command> {
    match drag_state.kind {
        DragKind::Shapes(shape_drag) => finish_shapes_drag(doc, &shape_drag, cursor_world),
        DragKind::Anchor(anchor_drag) => finish_anchor_drag(doc, &anchor_drag, cursor_world),
        DragKind::Handle(handle_drag) => finish_handle_drag(doc, &handle_drag, cursor_world)
            .map(|movement| Command::MoveHandle { movement }),
    }
}

fn start_shapes_drag(
    doc: &Document,
    selection: &crate::model::Selection,
    cursor_world: Vec2,
    hit: ShapeHit,
) -> Option<ShapesDragState> {
    let drag_all_selected = selection.contains(&SelItem::Shape(hit.shape_id));
    let shape_ids = if drag_all_selected {
        selected_shape_ids_for_drag(selection)
    } else {
        vec![hit.shape_id]
    };

    let mut shapes = Vec::new();
    let mut hit_index_hint = Some(hit.shape_index).filter(|index| {
        doc.shape_at(*index)
            .is_some_and(|shape| shape.id == hit.shape_id)
    });

    for shape_id in shape_ids {
        let index = if shape_id == hit.shape_id {
            hit_index_hint.take().or_else(|| doc.find_index(shape_id))?
        } else {
            doc.find_index(shape_id)?
        };

        let shape = doc.shape_at(index)?;
        if shape.id != shape_id {
            continue;
        }

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
    let shape = doc.shape_at(hit.shape_index)?;
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
        let Some(shape) = doc.shape_at_mut(dragged.index) else {
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
    let Some(shape) = doc.shape_at_mut(drag.shape_index) else {
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

fn finish_shapes_drag(
    doc: &mut Document,
    drag: &ShapesDragState,
    cursor_world: Vec2,
) -> Option<Command> {
    let Some(delta) = drag_delta(drag.start_cursor_world, cursor_world) else {
        let _did_restore = apply_shapes_drag_to_doc(doc, drag, Vec2::ZERO);
        return None;
    };
    let _did_restore = apply_shapes_drag_to_doc(doc, drag, Vec2::ZERO);

    let movements = drag
        .shapes
        .iter()
        .map(|dragged| ShapeMovement {
            shape_id: dragged.shape,
            delta,
        })
        .collect();

    Some(Command::MoveShapes { movements })
}

fn finish_anchor_drag(
    doc: &mut Document,
    drag: &AnchorDragState,
    cursor_world: Vec2,
) -> Option<Command> {
    let Some(delta) = drag_delta(drag.start_cursor_world, cursor_world) else {
        let _did_restore = apply_anchor_drag_to_doc(doc, drag, Vec2::ZERO);
        return None;
    };
    let _did_restore = apply_anchor_drag_to_doc(doc, drag, Vec2::ZERO);

    Some(Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: drag.shape,
            anchor_index: drag.anchor_index,
            original: drag.original_anchor.clone(),
            delta,
        },
    })
}

fn drag_delta(start_cursor_world: Vec2, cursor_world: Vec2) -> Option<Vec2> {
    let delta = cursor_world.sub(start_cursor_world);

    if delta.x.abs() <= f32::EPSILON && delta.y.abs() <= f32::EPSILON {
        return None;
    }

    Some(delta)
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
