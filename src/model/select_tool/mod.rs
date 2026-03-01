//! `SelectTool` FSM and input/state contracts.
//!
//! This module models manipulate-mode selection and drag state transitions as a
//! deterministic tool FSM emitting `ToolCommand` values.

mod drag;
mod selection;

use crate::model::tool::{EdgeMode, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolTransition};
use crate::model::{Document, Selection, ShapeId, Vec2};

use self::drag::{
    SelectDragStartInput, apply_drag_preview, finish_drag_command, restore_drag_preview, start_drag,
};
use self::selection::{can_drag_shape_bbox, selection_for_hit};

pub use self::drag::SelectDragState;

/// FSM state for manipulate interactions.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SelectToolState {
    /// No active manipulate gesture.
    #[default]
    Idle,
    /// A drag gesture is active.
    Dragging(SelectDragState),
    /// Reserved for marquee selection interaction.
    Marquee,
    /// Reserved for transform interaction.
    Transforming,
}

/// Hit result consumed by `SelectTool` pointer-down transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectPointerHit {
    /// Pointer-down hit on a handle.
    Handle(SelectHandleHit),
    /// Pointer-down hit on an anchor.
    Anchor(SelectAnchorHit),
    /// Pointer-down hit on a segment.
    Segment(SelectSegmentHit),
    /// Pointer-down hit on shape body / bounding box.
    Shape(SelectShapeHit),
    /// Pointer-down with no hit target.
    None,
}

/// Hit payload for a shape-level pointer interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectShapeHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
}

/// Hit payload for an anchor-level pointer interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectAnchorHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
    /// Anchor index in `shape.path.anchors`.
    pub anchor_index: usize,
}

/// Which handle was hit during pointer-down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectHandleHitKind {
    /// Incoming handle.
    In,
    /// Outgoing handle.
    Out,
}

/// Hit payload for a handle-level pointer interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectHandleHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
    /// Anchor index in `shape.path.anchors`.
    pub anchor_index: usize,
    /// Which handle endpoint was hit.
    pub kind: SelectHandleHitKind,
}

/// Hit payload for a segment-level pointer interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectSegmentHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
    /// Segment index in `shape.path.segments`.
    pub seg_index: usize,
}

/// Input context for one manipulate pointer-down transition.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectPointerDownInput {
    /// Document snapshot used for deterministic drag-start state creation.
    pub document: Document,
    /// Selection state before processing pointer-down.
    pub previous_selection: Selection,
    /// Pointer-down hit payload.
    pub hit: SelectPointerHit,
    /// Pointer position in world coordinates.
    pub cursor_world: Vec2,
    /// Whether Shift was held for additive/toggle selection behaviour.
    pub is_shift_held: bool,
}

/// Input context for one manipulate pointer-move transition.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectPointerMoveInput {
    /// Current select-tool state.
    pub state: SelectToolState,
    /// Pointer position in world coordinates.
    pub cursor_world: Vec2,
    /// Whether primary button is still pressed.
    pub has_primary_button: bool,
}

/// Input context for one manipulate pointer-up transition.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectPointerUpInput {
    /// Current select-tool state.
    pub state: SelectToolState,
    /// Pointer position in world coordinates.
    pub cursor_world: Vec2,
    /// Whether pointer-up was for primary button.
    pub is_primary_button: bool,
}

/// Manipulate-mode FSM.
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectTool;

impl Tool for SelectTool {
    fn transition(
        &self,
        current_mode: ToolMode,
        _current_edge_mode: EdgeMode,
        event: ToolInputEvent,
    ) -> ToolTransition {
        if current_mode != ToolMode::Manipulate {
            return ToolTransition::default();
        }

        match event {
            ToolInputEvent::SelectPointerDown { input } => on_pointer_down(input.as_ref()),
            ToolInputEvent::SelectPointerMove { input } => on_pointer_move(input.as_ref()),
            ToolInputEvent::SelectPointerUp { input } => on_pointer_up(input.as_ref()),
            _ => ToolTransition::default(),
        }
    }
}

/// Apply preview movement for the active drag state, if any.
pub fn apply_select_drag_preview(
    document: &mut Document,
    state: &SelectToolState,
    cursor_world: Vec2,
) -> bool {
    let SelectToolState::Dragging(drag_state) = state else {
        return false;
    };

    apply_drag_preview(document, drag_state, cursor_world)
}

/// Restore the preview-mutated document back to the drag start state.
pub fn restore_select_drag_preview(document: &mut Document, state: &SelectToolState) -> bool {
    let SelectToolState::Dragging(drag_state) = state else {
        return false;
    };

    restore_drag_preview(document, drag_state)
}

fn on_pointer_down(input: &SelectPointerDownInput) -> ToolTransition {
    let mut commands = Vec::new();

    let new_selection =
        selection_for_hit(&input.previous_selection, input.hit, input.is_shift_held);

    if new_selection != input.previous_selection {
        commands.push(ToolCommand::RecordSelectionChange {
            from: input.previous_selection.clone(),
            to: new_selection.clone(),
        });
        commands.push(ToolCommand::SetSelection(new_selection.clone()));
    }

    if input.is_shift_held {
        commands.push(ToolCommand::SetSelectToolState(SelectToolState::Idle));
        return ToolTransition::with_commands(commands);
    }

    let can_drag_bbox = can_drag_shape_bbox(&input.previous_selection, input.hit);
    let next_state = start_drag(
        &SelectDragStartInput::new(
            &input.document,
            &new_selection,
            input.cursor_world,
            can_drag_bbox,
        ),
        input.hit,
    )
    .map_or(SelectToolState::Idle, SelectToolState::Dragging);

    commands.push(ToolCommand::SetSelectToolState(next_state));
    ToolTransition::with_commands(commands)
}

fn on_pointer_move(input: &SelectPointerMoveInput) -> ToolTransition {
    if !input.has_primary_button {
        return ToolTransition::default();
    }

    if !matches!(input.state, SelectToolState::Dragging(_)) {
        return ToolTransition::default();
    }

    ToolTransition::with_commands([ToolCommand::PreviewSelectDrag {
        cursor_world: input.cursor_world,
    }])
}

fn on_pointer_up(input: &SelectPointerUpInput) -> ToolTransition {
    if !input.is_primary_button {
        return ToolTransition::default();
    }

    let SelectToolState::Dragging(drag_state) = &input.state else {
        return ToolTransition::default();
    };

    let mut commands = vec![ToolCommand::RestoreSelectDragPreview];

    if let Some(command) = finish_drag_command(drag_state, input.cursor_world) {
        commands.push(ToolCommand::ApplyDocumentCommand(Box::new(command)));
    }

    commands.push(ToolCommand::SetSelectToolState(SelectToolState::Idle));
    ToolTransition::with_commands(commands)
}
