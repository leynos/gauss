//! Gauss editor data model.
//!
//! This module contains editor-friendly representations of vector geometry
//! (paths, anchors, handles), document structure, selection, and viewport
//! transforms.
//!
//! It is intentionally independent of GPUI so it can be unit tested and reused
//! by alternative frontends.

pub mod action;
pub(crate) mod colour;
pub mod command;
pub mod document;
pub(crate) mod geometry;
pub mod key_context;
pub mod keybinding;
pub mod keystroke;
pub mod ops;
pub mod path;
pub mod resize_anchor;
pub mod selection;
pub mod viewport;

pub use action::{Action, ActionKind};
pub(crate) use colour::{format_hex_rgb, parse_hex_rgb};
pub use command::{Command, CommandInverse, DeletedShape, UserError, prepare_command};
pub use document::Document;
pub(crate) use geometry::{CubicSegment, cubic_point, shape_world_bounds};
pub use key_context::KeyContext;
pub use keybinding::{
    ActionBinding, bindings_for_action, bindings_for_context, default_bindings, primary_keystroke,
};
pub use keystroke::{Keystroke, Modifiers};
pub use ops::{DocChange, DocOp};
pub use path::{Anchor, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2};
pub use resize_anchor::ResizeAnchor;
pub use selection::{SelItem, Selection};
pub use viewport::Viewport;

#[cfg(test)]
mod ops_roundtrip_tests;
