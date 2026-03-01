//! Gauss editor data model.
//!
//! This module contains editor-friendly representations of vector geometry
//! (paths, anchors, handles), document structure, selection, viewport
//! transforms, and unified engine state.
//!
//! It is intentionally independent of GPUI so it can be unit tested and reused
//! by alternative frontends.

pub mod action;
pub(crate) mod colour;
pub mod command;
pub mod document;
pub mod engine_state;
pub(crate) mod geometry;
pub mod history;
pub mod key_context;
pub mod keybinding;
pub mod keystroke;
pub mod ops;
pub mod path;
pub(crate) mod pen_geometry;
pub mod resize_anchor;
pub mod resource_store;
pub mod select_tool;
pub mod selection;
pub mod style_store;
pub mod tool;
pub(crate) mod unique_string;
pub mod viewport;

pub use action::{Action, ActionKind};
pub(crate) use colour::{format_hex_rgb, parse_hex_rgb};
pub use command::{
    AnchorDeletion, AnchorDeletionResult, AnchorMovement, AnchorRestoration, AnchorRestorationKind,
    Command, CommandInverse, DeletedShape, HandleKind, HandleMovement, ReorderOp, SegmentChange,
    ShapeInsertion, ShapeMovement, ShapeReplacement, StyleChange, UserError, prepare_command,
};
pub use document::Document;
pub use engine_state::EngineState;
pub(crate) use geometry::{CubicSegment, cubic_point, shape_world_bounds};
pub use history::{DocumentUndoHistory, HistoryError};
pub use key_context::KeyContext;
pub use keybinding::{
    ActionBinding, bindings_for_action, bindings_for_context, default_bindings, primary_keystroke,
};
pub use keystroke::{Keystroke, Modifier, Modifiers};
pub use ops::{DocChange, DocOp};
pub use path::{
    Anchor, GaussAttribute, Paint, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2,
};
pub use resize_anchor::ResizeAnchor;
pub use resource_store::{
    Gradient, GradientId, GradientKind, GradientStop, LinearGradient, PatternId, PatternResource,
    RadialGradient, ResourceStore, SymbolId, SymbolResource,
};
pub use select_tool::{
    SelectAnchorHit, SelectHandleHit, SelectHandleHitKind, SelectPointerDownInput,
    SelectPointerHit, SelectPointerMoveInput, SelectPointerUpInput, SelectSegmentHit,
    SelectShapeHit, SelectTool, SelectToolState, apply_select_drag_preview,
    restore_select_drag_preview,
};
pub use selection::{SelItem, Selection};
pub use style_store::{NamedStyle, StyleId, StyleStore};
pub use tool::{
    EdgeMode, PenTool, PenToolActiveShape, PenToolClickInput, Tool, ToolCommand, ToolInputEvent,
    ToolMode, ToolModeFsm, ToolTransition,
};
pub use viewport::Viewport;

#[cfg(test)]
mod ops_roundtrip_tests;
#[cfg(test)]
mod pen_tool_tests;
#[cfg(test)]
mod resource_store_tests;
#[cfg(test)]
mod select_tool_drag_tests;
#[cfg(test)]
mod select_tool_preview_tests;
#[cfg(test)]
mod select_tool_tests;
#[cfg(test)]
mod tool_tests;
