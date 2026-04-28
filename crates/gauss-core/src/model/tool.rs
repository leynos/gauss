//! Tool and edge mode definitions plus the Tool FSM contract.
//!
//! This module defines the primary interaction modes for the editor, plus the
//! command-emitting finite-state-machine (FSM) boundary used by tool logic.
//! Tool mode determines which tool is active (for example, pen for drawing,
//! selection for manipulation). Edge mode determines how new path segments are
//! created.
//!
//! These types are GPUI-independent for testability and scripting.

use crate::model::Selection;
use crate::model::command::{Command, ShapeInsertion, ShapeReplacement};
use crate::model::path::{PaintStyle, Shape, ShapeId, Vec2};
use crate::model::pen_geometry::{
    append_anchor, close_shape, new_open_shape, segment_kind_for_edge_mode, should_close_path,
};
use crate::model::select_tool::{
    SelectPointerDownInput, SelectPointerMoveInput, SelectPointerUpInput, SelectToolState,
};

/// The active tool in the editor.
///
/// Tool mode determines how user input is interpreted. In Draw mode, clicks
/// place anchors to create paths. In Manipulate mode, clicks select and move
/// existing shapes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolMode {
    /// Draw mode: clicks place anchors to create new paths.
    #[default]
    Draw,
    /// Manipulate mode: clicks select and move existing shapes.
    Manipulate,
}

impl ToolMode {
    /// Return a human-readable label for this tool mode.
    ///
    /// Used for status line display and accessibility labels.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Manipulate => "Manipulate",
        }
    }
}

/// The edge mode for path segment creation.
///
/// Edge mode determines how new segments are connected when drawing paths.
/// Line mode creates straight segments. Bezier (auto) mode creates smooth
/// curves with automatically calculated handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeMode {
    /// Straight line segments between anchors.
    #[default]
    Line,
    /// Smooth curves with Catmull-Rom interpolated handles.
    BezierAuto,
}

impl EdgeMode {
    /// Return a human-readable label for this edge mode.
    ///
    /// Used for status line display and accessibility labels.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::BezierAuto => "Bezier (auto)",
        }
    }

    /// Return the opposite edge mode.
    ///
    /// Used for Tab key toggle behaviour.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Line => Self::BezierAuto,
            Self::BezierAuto => Self::Line,
        }
    }
}

/// Input events consumed by a tool state machine.
#[derive(Clone, Debug, PartialEq)]
pub enum ToolInputEvent {
    /// Activate draw mode, optionally selecting a specific edge mode.
    ActivateDraw {
        /// Optional edge mode to apply when entering draw mode.
        edge_mode: Option<EdgeMode>,
    },
    /// Activate manipulate mode.
    ActivateManipulate,
    /// Escape key pressed.
    EscapePressed,
    /// Toggle edge mode in draw context.
    ToggleEdgeMode,
    /// A close-path commit completed successfully.
    ClosePathCommitted,
    /// Draw-mode canvas click routed to the `PenTool` FSM.
    PenCanvasClicked {
        /// Input context snapshot for one canvas-click transition.
        input: Box<PenToolClickInput>,
    },
    /// Manipulate-mode pointer-down input for `SelectTool`.
    SelectPointerDown {
        /// Input context snapshot for one pointer-down transition.
        input: Box<SelectPointerDownInput>,
    },
    /// Manipulate-mode pointer-move input for `SelectTool`.
    SelectPointerMove {
        /// Input context snapshot for one pointer-move transition.
        input: Box<SelectPointerMoveInput>,
    },
    /// Manipulate-mode pointer-up input for `SelectTool`.
    SelectPointerUp {
        /// Input context snapshot for one pointer-up transition.
        input: Box<SelectPointerUpInput>,
    },
}

/// Command outputs emitted by tool FSMs.
///
/// Tool logic must emit commands and never mutate editor state directly.
#[derive(Clone, Debug, PartialEq)]
pub enum ToolCommand {
    /// Apply a document command through the command history pipeline.
    ApplyDocumentCommand(Box<Command>),
    /// Set active tool mode.
    SetToolMode(ToolMode),
    /// Set active edge mode.
    SetEdgeMode(EdgeMode),
    /// Set active path identity used by draw mode.
    SetActivePath(Option<ShapeId>),
    /// Set active selection.
    SetSelection(Selection),
    /// Record one selection history transition.
    RecordSelectionChange {
        /// Selection before the change.
        from: Selection,
        /// Selection after the change.
        to: Selection,
    },
    /// Set the current manipulate-tool FSM state.
    SetSelectToolState(SelectToolState),
    /// Apply preview updates for the active select drag state.
    PreviewSelectDrag {
        /// Cursor position in world coordinates.
        cursor_world: Vec2,
    },
    /// Restore the preview-mutated document back to drag start coordinates.
    RestoreSelectDragPreview,
}

/// Result of handling one tool input event.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolTransition {
    /// Ordered commands produced by the transition.
    pub commands: Vec<ToolCommand>,
}

impl ToolTransition {
    /// Build a transition from command outputs.
    #[must_use]
    pub fn with_commands(commands: impl IntoIterator<Item = ToolCommand>) -> Self {
        Self {
            commands: commands.into_iter().collect(),
        }
    }
}

/// Snapshot of the currently active draw path.
#[derive(Clone, Debug, PartialEq)]
pub struct PenToolActiveShape {
    /// Draw-order index of the active shape.
    pub index: usize,
    /// Shape snapshot used to build replacement commands.
    pub shape: Shape,
}

/// Input context for one `PenTool` canvas click transition.
#[derive(Clone, Debug, PartialEq)]
pub struct PenToolClickInput {
    /// Cursor position in world coordinates.
    pub cursor_world: Vec2,
    /// Current viewport zoom used for close-path snapping.
    pub zoom: f32,
    /// Current style used when starting a new shape.
    pub current_style: PaintStyle,
    /// Current active path ID (if any).
    pub active_path: Option<ShapeId>,
    /// Active-path shape snapshot when the path exists in the document.
    pub active_shape: Option<PenToolActiveShape>,
    /// Preallocated shape id for a new shape transition.
    pub next_shape_id: ShapeId,
    /// Current document length used as insert index for new shapes.
    pub document_len: usize,
    /// Snap radius in screen pixels for close-path detection.
    pub snap_radius_px: f32,
}

impl PenToolClickInput {
    /// Default close-path snap radius in pixels.
    pub const DEFAULT_SNAP_RADIUS_PX: f32 = 10.0;
}

/// Pen-tool finite-state machine for draw-click transitions.
#[derive(Clone, Copy, Debug, Default)]
pub struct PenTool;

impl PenTool {
    fn on_canvas_click(input: &PenToolClickInput, current_edge_mode: EdgeMode) -> ToolTransition {
        let Some(active_path) = input.active_path else {
            return ToolTransition::with_commands(start_new_shape_commands(input));
        };
        let Some(active_shape) = input
            .active_shape
            .as_ref()
            .filter(|snapshot| snapshot.shape.id == active_path)
        else {
            return ToolTransition::with_commands(recover_stale_active_path_commands(input));
        };

        if should_close_path(
            &active_shape.shape.path,
            input.cursor_world,
            input.zoom,
            input.snap_radius_px,
        ) {
            let old_shape = active_shape.shape.clone();
            let new_shape = close_shape(
                old_shape.clone(),
                segment_kind_for_edge_mode(current_edge_mode),
            );
            let command = Command::ClosePath {
                replacement: ShapeReplacement {
                    shape_index: active_shape.index,
                    old_shape,
                    new_shape,
                },
            };
            return ToolTransition::with_commands([
                ToolCommand::ApplyDocumentCommand(Box::new(command)),
                ToolCommand::SetToolMode(ToolMode::Manipulate),
                ToolCommand::SetActivePath(None),
            ]);
        }

        let old_shape = active_shape.shape.clone();
        let new_shape = append_anchor(old_shape.clone(), input.cursor_world, current_edge_mode);
        let command = Command::InsertAnchor {
            replacement: ShapeReplacement {
                shape_index: active_shape.index,
                old_shape,
                new_shape,
            },
        };
        ToolTransition::with_commands([ToolCommand::ApplyDocumentCommand(Box::new(command))])
    }
}

impl Tool for PenTool {
    fn transition(
        &self,
        current_mode: ToolMode,
        current_edge_mode: EdgeMode,
        event: ToolInputEvent,
    ) -> ToolTransition {
        if current_mode != ToolMode::Draw {
            return ToolTransition::default();
        }

        let ToolInputEvent::PenCanvasClicked { input } = event else {
            return ToolTransition::default();
        };

        Self::on_canvas_click(input.as_ref(), current_edge_mode)
    }
}

fn recover_stale_active_path_commands(input: &PenToolClickInput) -> Vec<ToolCommand> {
    let mut commands = vec![ToolCommand::SetActivePath(None)];
    commands.extend(start_new_shape_commands(input));
    commands
}

fn start_new_shape_commands(input: &PenToolClickInput) -> Vec<ToolCommand> {
    let shape = new_open_shape(
        input.next_shape_id,
        input.cursor_world,
        input.current_style.clone(),
    );
    let command = Command::InsertShape {
        insertion: ShapeInsertion {
            index: input.document_len,
            shape,
        },
    };
    vec![
        ToolCommand::ApplyDocumentCommand(Box::new(command)),
        ToolCommand::SetActivePath(Some(input.next_shape_id)),
    ]
}

/// Tool finite-state-machine contract.
///
/// Implementations are deterministic: given the same state snapshot and input
/// event, they must emit the same command sequence.
pub trait Tool {
    /// Handle one input event and return emitted commands.
    fn transition(
        &self,
        current_mode: ToolMode,
        current_edge_mode: EdgeMode,
        event: ToolInputEvent,
    ) -> ToolTransition;
}

/// FSM for draw/manipulate mode transitions.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToolModeFsm;

impl ToolModeFsm {
    fn handle_activate_draw(
        current_mode: ToolMode,
        current_edge_mode: EdgeMode,
        requested: Option<EdgeMode>,
    ) -> ToolTransition {
        let mut commands = Vec::new();
        if current_mode != ToolMode::Draw {
            commands.push(ToolCommand::SetToolMode(ToolMode::Draw));
        }
        if let Some(next) = Self::compute_next_edge_mode(current_edge_mode, requested) {
            Self::push_edge_mode_command(&mut commands, next);
        }
        ToolTransition::with_commands(commands)
    }

    fn compute_next_edge_mode(current: EdgeMode, requested: Option<EdgeMode>) -> Option<EdgeMode> {
        requested.filter(|&next| next != current)
    }

    fn push_edge_mode_command(out: &mut Vec<ToolCommand>, next: EdgeMode) {
        out.push(ToolCommand::SetEdgeMode(next));
    }
}

impl Tool for ToolModeFsm {
    fn transition(
        &self,
        current_mode: ToolMode,
        current_edge_mode: EdgeMode,
        event: ToolInputEvent,
    ) -> ToolTransition {
        match event {
            ToolInputEvent::ActivateDraw { edge_mode } => {
                Self::handle_activate_draw(current_mode, current_edge_mode, edge_mode)
            }
            ToolInputEvent::ActivateManipulate => ToolTransition::with_commands([
                ToolCommand::SetToolMode(ToolMode::Manipulate),
                ToolCommand::SetActivePath(None),
            ]),
            ToolInputEvent::EscapePressed => match current_mode {
                ToolMode::Draw => ToolTransition::with_commands([
                    ToolCommand::SetToolMode(ToolMode::Manipulate),
                    ToolCommand::SetActivePath(None),
                ]),
                ToolMode::Manipulate => {
                    ToolTransition::with_commands([ToolCommand::SetToolMode(ToolMode::Draw)])
                }
            },
            ToolInputEvent::ToggleEdgeMode => {
                if current_mode != ToolMode::Draw {
                    return ToolTransition::default();
                }
                ToolTransition::with_commands([ToolCommand::SetEdgeMode(
                    current_edge_mode.toggle(),
                )])
            }
            ToolInputEvent::ClosePathCommitted => {
                if current_mode != ToolMode::Draw {
                    return ToolTransition::default();
                }
                ToolTransition::with_commands([
                    ToolCommand::SetToolMode(ToolMode::Manipulate),
                    ToolCommand::SetActivePath(None),
                ])
            }
            ToolInputEvent::PenCanvasClicked { .. }
            | ToolInputEvent::SelectPointerDown { .. }
            | ToolInputEvent::SelectPointerMove { .. }
            | ToolInputEvent::SelectPointerUp { .. } => ToolTransition::default(),
        }
    }
}
