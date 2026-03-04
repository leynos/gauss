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
struct SelectDragShapeSnapshot {
    id: ShapeId,
    anchors: Vec<Anchor>,
}

/// Lightweight document snapshot used to initialise select-tool drags.
///
/// Drag-start logic only needs shape identity, ordering, and anchor geometry,
/// so this snapshot avoids carrying full style/metadata document state.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectDragDocumentSnapshot {
    shapes: Vec<SelectDragShapeSnapshot>,
}

impl SelectDragDocumentSnapshot {
    /// Capture drag-start data from a document.
    #[must_use]
    pub fn from_document(doc: &Document) -> Self {
        let shapes = doc
            .iter_in_draw_order()
            .map(|shape| SelectDragShapeSnapshot {
                id: shape.id,
                anchors: shape.path.anchors.clone(),
            })
            .collect();
        Self { shapes }
    }

    fn find_index(&self, id: ShapeId) -> Option<usize> {
        self.shapes.iter().position(|shape| shape.id == id)
    }

    fn shape_at(&self, index: usize) -> Option<&SelectDragShapeSnapshot> {
        self.shapes.get(index)
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
    /// Draw-order index captured at drag start.
    pub shape_index: usize,
    /// Anchor index captured at drag start.
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
    /// Draw-order index captured at drag start.
    pub shape_index: usize,
    /// Anchor index captured at drag start.
    pub anchor_index: usize,
    /// Which handle endpoint is being dragged.
    pub kind: SelectHandleHitKind,
    /// Cursor position at drag start.
    pub start_cursor_world: Vec2,
    /// Original handle position used for restore and command creation.
    pub original_handle: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SelectDragStartParams {
    pub cursor_world: Vec2,
    pub can_drag_shape_bbox: bool,
}

pub(super) fn start_drag(
    snapshot: &SelectDragDocumentSnapshot,
    selection: &crate::model::Selection,
    start: SelectDragStartParams,
    hit: SelectPointerHit,
) -> Option<SelectDragState> {
    match hit {
        SelectPointerHit::Handle(handle_hit) => {
            start_handle_drag(snapshot, handle_hit, start.cursor_world).map(SelectDragState::Handle)
        }
        SelectPointerHit::Anchor(anchor_hit) => {
            start_anchor_drag(snapshot, anchor_hit, start.cursor_world).map(SelectDragState::Anchor)
        }
        SelectPointerHit::Segment(segment_hit) => start_shapes_drag(
            snapshot,
            selection,
            start.cursor_world,
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
            if !start.can_drag_shape_bbox {
                return None;
            }
            start_shapes_drag(
                snapshot,
                selection,
                start.cursor_world,
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
    let delta = match drag_state {
        SelectDragState::Shapes(drag) => drag_delta(drag.start_cursor_world, cursor_world),
        SelectDragState::Anchor(drag) => drag_delta(drag.start_cursor_world, cursor_world),
        SelectDragState::Handle(drag) => drag_delta(drag.start_cursor_world, cursor_world),
    };

    if is_zero_delta(delta) {
        return None;
    }

    Some(match drag_state {
        SelectDragState::Shapes(shape_drag) => finish_shapes_drag_command(shape_drag, delta),
        SelectDragState::Anchor(anchor_drag) => finish_anchor_drag_command(anchor_drag, delta),
        SelectDragState::Handle(handle_drag) => Command::MoveHandle {
            movement: finish_handle_drag_command(handle_drag, delta),
        },
    })
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
    snapshot: &SelectDragDocumentSnapshot,
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
        snapshot
            .shape_at(*index)
            .is_some_and(|shape| shape.id == hit.shape_id)
    });

    for shape_id in shape_ids {
        let index = if shape_id == hit.shape_id {
            hit_index_hint
                .take()
                .or_else(|| snapshot.find_index(shape_id))?
        } else {
            snapshot.find_index(shape_id)?
        };

        let shape = snapshot.shape_at(index)?;
        if shape.id != shape_id {
            continue;
        }

        shapes.push(DraggedShape {
            shape: shape_id,
            index,
            original_anchors: shape.anchors.clone(),
        });
    }

    (!shapes.is_empty()).then_some(ShapesDragState {
        start_cursor_world: cursor_world,
        shapes,
    })
}

fn start_anchor_drag(
    snapshot: &SelectDragDocumentSnapshot,
    hit: SelectAnchorHit,
    cursor_world: Vec2,
) -> Option<AnchorDragState> {
    let shape = snapshot.shape_at(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }
    let anchor = shape.anchors.get(hit.anchor_index)?.clone();

    Some(AnchorDragState {
        shape: hit.shape_id,
        shape_index: hit.shape_index,
        anchor_index: hit.anchor_index,
        start_cursor_world: cursor_world,
        original_anchor: anchor,
    })
}

fn start_handle_drag(
    snapshot: &SelectDragDocumentSnapshot,
    hit: SelectHandleHit,
    cursor_world: Vec2,
) -> Option<HandleDragState> {
    let shape = snapshot.shape_at(hit.shape_index)?;
    if shape.id != hit.shape_id {
        return None;
    }

    let anchor = shape.anchors.get(hit.anchor_index)?;
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

fn finish_shapes_drag_command(drag: &ShapesDragState, delta: Vec2) -> Command {
    let movements = drag
        .shapes
        .iter()
        .map(|dragged| ShapeMovement {
            shape_id: dragged.shape,
            delta,
        })
        .collect();

    Command::MoveShapes { movements }
}

fn finish_anchor_drag_command(drag: &AnchorDragState, delta: Vec2) -> Command {
    Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: drag.shape,
            anchor_index: drag.anchor_index,
            original: drag.original_anchor.clone(),
            delta,
        },
    }
}

const fn finish_handle_drag_command(drag: &HandleDragState, delta: Vec2) -> HandleMovement {
    let to = drag.original_handle.add(delta);
    HandleMovement {
        shape_id: drag.shape,
        anchor_index: drag.anchor_index,
        kind: match drag.kind {
            SelectHandleHitKind::In => HandleKind::In,
            SelectHandleHitKind::Out => HandleKind::Out,
        },
        from: Some(drag.original_handle),
        to: Some(to),
    }
}

const fn drag_delta(start_cursor_world: Vec2, cursor_world: Vec2) -> Vec2 {
    cursor_world.sub(start_cursor_world)
}

const fn is_zero_delta(delta: Vec2) -> bool {
    delta.x.abs() <= f32::EPSILON && delta.y.abs() <= f32::EPSILON
}
