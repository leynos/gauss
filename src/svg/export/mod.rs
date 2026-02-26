//! SVG export for the Gauss document model.
#![expect(
    clippy::float_arithmetic,
    reason = "SVG export needs floating-point conversions for geometry"
)]
mod defs;
mod shape_metadata;
use crate::model::{
    Document, GradientId, Paint, PatternId, ResourceStore, SegmentKind, Shape, format_hex_rgb,
};
use crate::svg::metadata::gauss_namespace_declaration;
use std::fmt::{Arguments, Write as _};
/// Errors returned by checked SVG export entry points such as
/// [`export_svg_with_resources_checked`],
/// [`export_svg_with_resources_web_ready_checked`], and
/// [`export_svg_with_metadata_checked`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgExportError {
    /// A shape references a gradient ID that does not exist in `ResourceStore`.
    MissingGradientReference(GradientId),
    /// A shape references a pattern ID that does not exist in `ResourceStore`.
    MissingPatternReference(PatternId),
}
impl std::fmt::Display for SvgExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingGradientReference(id) => {
                write!(f, "shape references missing gradient resource '{id:?}'")
            }
            Self::MissingPatternReference(id) => {
                write!(f, "shape references missing pattern resource '{id:?}'")
            }
        }
    }
}
impl std::error::Error for SvgExportError {}
/// Export a document to SVG without shared resources.
#[must_use]
pub fn export_svg(doc: &Document, canvas_size: CanvasSize) -> String {
    export_svg_with_resources(doc, &ResourceStore::new(), canvas_size)
}
/// Export a document to an SVG string with explicit shared resources.
#[must_use]
pub fn export_svg_with_resources(
    doc: &Document,
    resources: &ResourceStore,
    canvas_size: CanvasSize,
) -> String {
    export_svg_with_options(ExportOptions::new(doc, resources, canvas_size))
}
/// Export a document to SVG with shared resources.
/// # Errors
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub fn export_svg_with_resources_checked(
    doc: &Document,
    resources: &ResourceStore,
    canvas_size: CanvasSize,
) -> Result<String, SvgExportError> {
    export_svg_with_options_checked(ExportOptions::new(doc, resources, canvas_size))
}
/// Export a document for web usage, stripping all Gauss metadata.
#[must_use]
pub fn export_svg_with_resources_web_ready(
    doc: &Document,
    resources: &ResourceStore,
    canvas_size: CanvasSize,
) -> String {
    export_svg_with_options(ExportOptions::new(doc, resources, canvas_size).web_ready())
}
/// Export a web-ready SVG and validate resource references.
/// # Errors
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub fn export_svg_with_resources_web_ready_checked(
    doc: &Document,
    resources: &ResourceStore,
    canvas_size: CanvasSize,
) -> Result<String, SvgExportError> {
    export_svg_with_options_checked(ExportOptions::new(doc, resources, canvas_size).web_ready())
}

/// Canvas dimensions used by SVG export operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasSize {
    /// Canvas width in document units.
    pub width: f32,
    /// Canvas height in document units.
    pub height: f32,
}
impl CanvasSize {
    /// Create a new canvas size.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Metadata policy for SVG export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportMode {
    /// Preserve Gauss metadata and namespace output.
    GaussWithMetadata,
    /// Strip Gauss metadata and namespace output for web publishing.
    WebReady,
}
impl ExportMode {
    #[must_use]
    const fn includes_gauss_metadata(self) -> bool {
        matches!(self, Self::GaussWithMetadata)
    }
}
/// Export options for metadata-aware SVG export.
#[derive(Clone, Copy)]
pub struct ExportOptions<'a> {
    /// Document to export.
    pub doc: &'a Document,
    /// Shared resources used by paint and defs export.
    pub resources: &'a ResourceStore,
    /// Canvas dimensions in document units.
    pub canvas_size: CanvasSize,
    /// Optional raw metadata block content.
    /// Ignored when [`Self::mode`] is [`ExportMode::WebReady`].
    pub metadata_block: Option<&'a str>,
    /// Metadata policy for this export operation.
    pub mode: ExportMode,
}
impl<'a> ExportOptions<'a> {
    /// Create export options with no metadata block.
    #[must_use]
    pub const fn new(
        doc: &'a Document,
        resources: &'a ResourceStore,
        canvas_size: CanvasSize,
    ) -> Self {
        Self {
            doc,
            resources,
            canvas_size,
            metadata_block: None,
            mode: ExportMode::GaussWithMetadata,
        }
    }
    /// Attach metadata block content that should be preserved verbatim.
    #[must_use]
    pub const fn with_metadata_block(mut self, metadata_block: &'a str) -> Self {
        self.metadata_block = Some(metadata_block);
        self
    }
    /// Mark this export as web-ready (strip Gauss metadata and metadata blocks).
    #[must_use]
    pub const fn web_ready(mut self) -> Self {
        self.mode = ExportMode::WebReady;
        self
    }
}
/// Export a document according to [`ExportOptions`].
#[must_use]
pub(crate) fn export_svg_with_options(options: ExportOptions<'_>) -> String {
    export_svg_inner(options)
}

/// Export a document according to [`ExportOptions`] with resource validation.
///
/// # Errors
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub(crate) fn export_svg_with_options_checked(
    options: ExportOptions<'_>,
) -> Result<String, SvgExportError> {
    validate_resource_references(options.doc, options.resources)?;
    Ok(export_svg_inner(options))
}
/// Export a document to an SVG string with metadata block preservation.
///
/// This helper always forces [`ExportOptions::mode`] to
/// [`ExportMode::GaussWithMetadata`] while preserving all other fields from
/// `options`.
#[must_use]
pub fn export_svg_with_metadata(options: ExportOptions<'_>) -> String {
    export_svg_with_options(ExportOptions {
        mode: ExportMode::GaussWithMetadata,
        ..options
    })
}
fn export_svg_inner(options: ExportOptions<'_>) -> String {
    let preserve_gauss_metadata = options.mode.includes_gauss_metadata();
    let mut out = String::new();
    write_svg_header(&mut out, options.canvas_size, preserve_gauss_metadata);
    defs::write_defs(&mut out, options.resources, preserve_gauss_metadata);
    if preserve_gauss_metadata {
        write_metadata_block(&mut out, options.metadata_block);
    }
    for shape in options.doc.iter_in_draw_order() {
        write_shape_path(&mut out, options.resources, shape, preserve_gauss_metadata);
    }
    out.push_str("</svg>\n");
    out
}
/// Export a document to SVG with metadata block preservation.
///
/// This helper always forces [`ExportOptions::mode`] to
/// [`ExportMode::GaussWithMetadata`] while preserving all other fields from
/// `options`.
///
/// # Errors
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub fn export_svg_with_metadata_checked(
    options: ExportOptions<'_>,
) -> Result<String, SvgExportError> {
    export_svg_with_options_checked(ExportOptions {
        mode: ExportMode::GaussWithMetadata,
        ..options
    })
}
#[must_use]
pub(super) const fn opacity_from_alpha(alpha: u8) -> f32 {
    (alpha as f32) / 255.0
}
fn write_svg_header(out: &mut String, canvas_size: CanvasSize, include_gauss_namespace: bool) {
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    let gauss_attr = if include_gauss_namespace {
        let gauss_namespace = gauss_namespace_declaration();
        format!(" {gauss_namespace}")
    } else {
        String::new()
    };
    write_fmt(
        out,
        format_args!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"{gauss_attr} width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#,
            gauss_attr = gauss_attr,
            width = canvas_size.width,
            height = canvas_size.height,
        ),
    );
    out.push('\n');
}
fn write_metadata_block(out: &mut String, metadata_block: Option<&str>) {
    let Some(content) = metadata_block else {
        return;
    };
    out.push_str("<metadata>");
    out.push_str(content);
    if !content.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("</metadata>\n");
}
fn write_shape_path(
    out: &mut String,
    resources: &ResourceStore,
    shape: &Shape,
    include_gauss_metadata: bool,
) {
    let Some(path_data) = build_path_data(shape) else {
        return;
    };
    let (stroke_attr, stroke_opacity) = format_paint(shape.style.stroke, resources);
    let (fill_attr, fill_opacity) = format_paint(shape.style.fill, resources);
    write_fmt(
        out,
        format_args!(
            r#"<path d="{path_data}" stroke="{stroke_attr}" stroke-width="{stroke_width}" fill="{fill_attr}""#,
            stroke_width = shape.style.stroke_width
        ),
    );
    write_optional_opacity(out, "stroke-opacity", &stroke_attr, stroke_opacity);
    write_optional_opacity(out, "fill-opacity", &fill_attr, fill_opacity);
    if include_gauss_metadata {
        shape_metadata::write_shape_gauss_metadata(out, shape);
    }
    out.push_str(" />\n");
}
fn build_path_data(shape: &Shape) -> Option<String> {
    let mut d = String::new();
    let first = shape.path.anchors.first()?;
    write_fmt(&mut d, format_args!("M {} {}", first.pos.x, first.pos.y));
    for (index, kind) in shape.path.segments.iter().enumerate() {
        let Some(start) = shape.path.anchors.get(index) else {
            break;
        };
        let Some(end) = shape.path.anchors.get(index + 1) else {
            break;
        };
        match kind {
            SegmentKind::Line => {
                write_fmt(&mut d, format_args!(" L {} {}", end.pos.x, end.pos.y));
            }
            SegmentKind::Cubic => {
                let c1 = start.handle_out.unwrap_or(start.pos);
                let c2 = end.handle_in.unwrap_or(end.pos);
                write_fmt(
                    &mut d,
                    format_args!(
                        " C {} {} {} {} {} {}",
                        c1.x, c1.y, c2.x, c2.y, end.pos.x, end.pos.y
                    ),
                );
            }
        }
    }
    if shape.path.closed {
        if shape.path.closing_segment == SegmentKind::Cubic
            && let (Some(last_anchor), Some(first_anchor)) =
                (shape.path.anchors.last(), shape.path.anchors.first())
        {
            let c1 = last_anchor.handle_out.unwrap_or(last_anchor.pos);
            let c2 = first_anchor.handle_in.unwrap_or(first_anchor.pos);
            write_fmt(
                &mut d,
                format_args!(
                    " C {} {} {} {} {} {}",
                    c1.x, c1.y, c2.x, c2.y, first_anchor.pos.x, first_anchor.pos.y
                ),
            );
        }
        d.push_str(" Z");
    }
    Some(d)
}
fn format_paint(paint: Paint, resources: &ResourceStore) -> (String, f32) {
    match paint {
        Paint::None => ("none".to_owned(), 1.0),
        Paint::Solid(colour) => (format_hex_rgb(colour), opacity_from_alpha(colour.a)),
        Paint::Gradient { id, opacity } => resources.gradient(id).map_or_else(
            || ("none".to_owned(), 1.0),
            |gradient| {
                (
                    format!("url(#{})", gradient.svg_id),
                    opacity_from_alpha(opacity),
                )
            },
        ),
        Paint::Pattern { id, opacity } => resources.pattern(id).map_or_else(
            || ("none".to_owned(), 1.0),
            |pattern| {
                (
                    format!("url(#{})", pattern.svg_id),
                    opacity_from_alpha(opacity),
                )
            },
        ),
    }
}
fn write_optional_opacity(out: &mut String, attr: &str, paint: &str, opacity: f32) {
    const OPACITY_EPSILON: f32 = 1.0e-6;
    if paint == "none" {
        return;
    }
    if opacity >= 1.0 - OPACITY_EPSILON {
        return;
    }
    write_fmt(out, format_args!(r#" {attr}="{opacity:.4}""#));
}
fn validate_resource_references(
    doc: &Document,
    resources: &ResourceStore,
) -> Result<(), SvgExportError> {
    for shape in doc.iter_in_draw_order() {
        validate_paint_reference(shape.style.stroke, resources)?;
        validate_paint_reference(shape.style.fill, resources)?;
    }
    Ok(())
}
fn validate_paint_reference(paint: Paint, resources: &ResourceStore) -> Result<(), SvgExportError> {
    match paint {
        Paint::Gradient { id, .. } if resources.gradient(id).is_none() => {
            Err(SvgExportError::MissingGradientReference(id))
        }
        Paint::Pattern { id, .. } if resources.pattern(id).is_none() => {
            Err(SvgExportError::MissingPatternReference(id))
        }
        Paint::None | Paint::Solid(_) | Paint::Gradient { .. } | Paint::Pattern { .. } => Ok(()),
    }
}
pub(super) fn write_fmt(out: &mut String, args: Arguments<'_>) {
    if out.write_fmt(args).is_err() {
        // `String` implements `fmt::Write` without failing, so this is
        // unreachable in practice.
    }
}
#[cfg(test)]
mod metadata_tests;
#[cfg(test)]
mod tests;
