//! Tool and edge mode definitions plus the Tool FSM contract.
//!
//! This module defines the primary interaction modes for the editor, plus the
//! command-emitting finite-state-machine (FSM) boundary used by tool logic.
//! Tool mode determines which tool is active (for example, pen for drawing,
//! selection for manipulation). Edge mode determines how new path segments are
//! created.
//!
//! These types are GPUI-independent for testability and scripting.

use crate::model::command::Command;
use crate::model::path::ShapeId;

/// The active tool in the editor.
///
/// Tool mode determines how user input is interpreted. In Draw mode, clicks
/// place anchors to create paths. In Manipulate mode, clicks select and move
/// existing shapes.
///
/// # Examples
///
/// ```rust
/// use gauss::model::ToolMode;
///
/// let mode = ToolMode::Draw;
/// assert_eq!(mode.label(), "Draw");
/// ```
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
///
/// # Examples
///
/// ```rust
/// use gauss::model::EdgeMode;
///
/// let mode = EdgeMode::Line;
/// assert_eq!(mode.label(), "Line");
/// assert_eq!(mode.toggle(), EdgeMode::BezierAuto);
/// ```
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Tool for ToolModeFsm {
    fn transition(
        &self,
        current_mode: ToolMode,
        current_edge_mode: EdgeMode,
        event: ToolInputEvent,
    ) -> ToolTransition {
        match event {
            ToolInputEvent::ActivateDraw { edge_mode } => {
                let mut commands = Vec::new();
                if current_mode != ToolMode::Draw {
                    commands.push(ToolCommand::SetToolMode(ToolMode::Draw));
                }
                if let Some(next_edge_mode) = edge_mode
                    && next_edge_mode != current_edge_mode
                {
                    commands.push(ToolCommand::SetEdgeMode(next_edge_mode));
                }
                ToolTransition::with_commands(commands)
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
        }
    }
}
