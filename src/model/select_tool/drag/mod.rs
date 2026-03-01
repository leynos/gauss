//! Drag-state helpers for `SelectTool`.
//!
//! Drag preview updates are deterministic and reversible; commit commands are
//! emitted separately so undo history records one step per gesture.

use crate::model::{
    Anchor, AnchorMovement, Command, Document, HandleKind, HandleMovement, ShapeId, ShapeMovement,
    Vec2,
};

use super::{
    SelectAnchorHit, SelectHandleHit, SelectHandleHitKind, SelectPointerHit, SelectShapeHit,
    selection::selected_shape_ids_for_drag,
};

mod preview;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SelectDragStartInput<'a> {
    doc: &'a Document,
    selection: &'a crate::model::Selection,
    cursor_world: Vec2,
    can_drag_shape_bbox: bool,
}

impl<'a> SelectDragStartInput<'a> {
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

/// Concrete drag states carried while `SelectToolState::Dragging` is active.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectDragState {
    /// Dragging one or more whole shapes.
    Shapes(ShapesDragState),
    /// Dragging one anchor (and attached handles).
    Anchor(AnchorDragState),
    /// Dragging a single handle endpoint.
    Handle(HandleDragState),
}

/// Drag state for moving one or more selected shapes.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapesDragState {
    /// Cursor position at drag start.
    pub start_cursor_world: Vec2,
    /// Snapshot entries for all dragged shapes.
    pub shapes: Vec<DraggedShape>,
}

/// Snapshot for one shape participating in a shapes drag.
#[derive(Clone, Debug, PartialEq)]
pub struct DraggedShape {
    /// Stable shape identifier.
    pub shape: ShapeId,
    /// Draw-order index captured at drag start.
    pub index: usize,
    /// Original anchor geometry used for preview and restore.
    pub original_anchors: Vec<Anchor>,
}

/// Drag state for moving one anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorDragState {
    /// Stable shape identifier.
    pub shape: ShapeId,
    pub shape_index: usize,
    pub anchor_index: usize,
    /// Cursor position at drag start.
    pub start_cursor_world: Vec2,
    /// Original anchor geometry for restore and command creation.
    pub original_anchor: Anchor,
}

/// Drag state for moving one handle endpoint.
#[derive(Clone, Debug, PartialEq)]
pub struct HandleDragState {
    /// Stable shape identifier.
    pub shape: ShapeId,
    pub shape_index: usize,
    pub anchor_index: usize,
    /// Which handle endpoint is being dragged.
    pub kind: SelectHandleHitKind,
    /// Cursor position at drag start.
    pub start_cursor_world: Vec2,
    /// Original handle position used for restore and command creation.
    pub original_handle: Vec2,
}

pub(super) fn start_drag(
    input: &SelectDragStartInput<'_>,
    hit: SelectPointerHit,
) -> Option<SelectDragState> {
    match hit {
        SelectPointerHit::Handle(handle_hit) => {
            start_handle_drag(input.doc, handle_hit, input.cursor_world)
                .map(SelectDragState::Handle)
        }
        SelectPointerHit::Anchor(anchor_hit) => {
            start_anchor_drag(input.doc, anchor_hit, input.cursor_world)
                .map(SelectDragState::Anchor)
        }
        SelectPointerHit::Segment(segment_hit) => start_shapes_drag(
            input.doc,
            input.selection,
            input.cursor_world,
            SelectShapeHit {
                shape_index: segment_hit.shape_index,
                shape_id: segment_hit.shape_id,
            },
        )
        .map(SelectDragState::Shapes),
        SelectPointerHit::Shape(SelectShapeHit {
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
                SelectShapeHit {
                    shape_index,
                    shape_id,
                },
            )
            .map(SelectDragState::Shapes)
        }
        SelectPointerHit::None => None,
    }
}

pub(super) fn finish_drag_command(
    drag_state: &SelectDragState,
    cursor_world: Vec2,
) -> Option<Command> {
    match drag_state {
        SelectDragState::Shapes(shape_drag) => finish_shapes_drag_command(shape_drag, cursor_world),
        SelectDragState::Anchor(anchor_drag) => {
            finish_anchor_drag_command(anchor_drag, cursor_world)
        }
        SelectDragState::Handle(handle_drag) => {
            finish_handle_drag_command(handle_drag, cursor_world)
                .map(|movement| Command::MoveHandle { movement })
        }
    }
}

pub(super) fn apply_drag_preview(
    doc: &mut Document,
    drag_state: &SelectDragState,
    cursor_world: Vec2,
) -> bool {
    preview::apply_drag_preview(doc, drag_state, cursor_world)
}

pub(super) fn restore_drag_preview(doc: &mut Document, drag_state: &SelectDragState) -> bool {
    preview::restore_drag_preview(doc, drag_state)
}

fn start_shapes_drag(
    doc: &Document,
    selection: &crate::model::Selection,
    cursor_world: Vec2,
    hit: SelectShapeHit,
) -> Option<ShapesDragState> {
    let drag_all_selected = selection.contains(&crate::model::SelItem::Shape(hit.shape_id));
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
    hit: SelectAnchorHit,
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

fn start_handle_drag(
    doc: &Document,
    hit: SelectHandleHit,
    cursor_world: Vec2,
) -> Option<HandleDragState> {
    let shape = doc.shape_at(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }

    let anchor = shape.path.anchors.get(hit.anchor_index)?;
    let original_handle = match hit.kind {
        SelectHandleHitKind::In => anchor.handle_in?,
        SelectHandleHitKind::Out => anchor.handle_out?,
    };

    Some(HandleDragState {
        shape: hit.shape_id,
        shape_index: hit.shape_index,
        anchor_index: hit.anchor_index,
        kind: hit.kind,
        start_cursor_world: cursor_world,
        original_handle,
    })
}

fn finish_shapes_drag_command(drag: &ShapesDragState, cursor_world: Vec2) -> Option<Command> {
    let delta = drag_delta(drag.start_cursor_world, cursor_world)?;

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

fn finish_anchor_drag_command(drag: &AnchorDragState, cursor_world: Vec2) -> Option<Command> {
    let delta = drag_delta(drag.start_cursor_world, cursor_world)?;

    Some(Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: drag.shape,
            anchor_index: drag.anchor_index,
            original: drag.original_anchor.clone(),
            delta,
        },
    })
}

fn finish_handle_drag_command(
    drag: &HandleDragState,
    cursor_world: Vec2,
) -> Option<HandleMovement> {
    let delta = drag_delta(drag.start_cursor_world, cursor_world)?;

    let to = drag.original_handle.add(delta);
    Some(HandleMovement {
        shape_id: drag.shape,
        anchor_index: drag.anchor_index,
        kind: match drag.kind {
            SelectHandleHitKind::In => HandleKind::In,
            SelectHandleHitKind::Out => HandleKind::Out,
        },
        from: Some(drag.original_handle),
        to: Some(to),
    })
}

fn drag_delta(start_cursor_world: Vec2, cursor_world: Vec2) -> Option<Vec2> {
    let delta = cursor_world.sub(start_cursor_world);

    if delta.x.abs() <= f32::EPSILON && delta.y.abs() <= f32::EPSILON {
        return None;
    }

    Some(delta)
}
