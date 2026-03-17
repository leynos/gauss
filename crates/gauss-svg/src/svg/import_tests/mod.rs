//! Tests for SVG import.
//!
//! These tests validate that we can import a small SVG subset and that
//! Gauss-exported SVG round-trips through the importer.

use crate::{
    model::{
        Anchor, Document, Gradient, GradientId, GradientKind, GradientStop, LinearGradient, Paint,
        PaintStyle, PathGeom, PatternId, PatternResource, ResourceStore, Rgba, SegmentKind, Shape,
        ShapeId, SymbolResource, Vec2,
    },
    svg::{
        export::{CanvasSize, export_svg_with_resources},
        import::{SvgImportError, import_svg, import_svg_with_resources},
        metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX},
    },
};
use rstest::rstest;

mod basic;
mod metadata;
mod namespaces;
mod resources;
