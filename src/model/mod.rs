//! Gauss editor data model.
//!
//! This module contains editor-friendly representations of vector geometry
//! (paths, anchors, handles), document structure, selection, and viewport
//! transforms.
//!
//! It is intentionally independent of GPUI so it can be unit tested and reused
//! by alternative frontends.

pub(crate) mod colour;
pub mod document;
pub(crate) mod geometry;
pub mod ops;
pub mod path;
pub mod selection;
pub mod viewport;

pub(crate) use colour::{format_hex_rgb, parse_hex_rgb};
pub use document::Document;
pub(crate) use geometry::{CubicSegment, cubic_point, shape_world_bounds};
pub use ops::{DocChange, DocOp};
pub use path::{Anchor, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2};
pub use selection::{SelItem, Selection};
pub use viewport::Viewport;

#[cfg(test)]
mod ops_roundtrip_tests;
