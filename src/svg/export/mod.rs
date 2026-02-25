//! SVG export for the Gauss document model.
#![expect(
    clippy::float_arithmetic,
    reason = "SVG export needs floating-point conversions for geometry"
)]
mod defs;
use std::fmt::{Arguments, Write as _};
use slotmap::Key;
use crate::model::{
    Document, GradientId, Paint, PatternId, ResourceStore, SegmentKind, Shape, format_hex_rgb,
};
use crate::svg::metadata::{gauss_namespace_declaration, shape_id_to_hex};
/// Errors returned by [`export_svg_with_resources_checked`].
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
pub fn export_svg(doc: &Document, canvas_width: f32, canvas_height: f32) -> String {
    export_svg_with_resources(doc, &ResourceStore::new(), canvas_width, canvas_height)
}
/// Export a document to an SVG string with explicit shared resources.
#[must_use]
pub fn export_svg_with_resources(
    doc: &Document,
    resources: &ResourceStore,
    canvas_width: f32,
    canvas_height: f32,
) -> String {
    export_svg_with_metadata(ExportOptions::new(
        doc,
        resources,
        canvas_width,
        canvas_height,
    ))
}
/// Export a document to SVG with shared resources.
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub fn export_svg_with_resources_checked(
    doc: &Document,
    resources: &ResourceStore,
    canvas_width: f32,
    canvas_height: f32,
) -> Result<String, SvgExportError> {
    export_svg_with_metadata_checked(ExportOptions::new(
        doc,
        resources,
        canvas_width,
        canvas_height,
    ))
}
/// Export a document for web usage, stripping all Gauss metadata.
#[must_use]
pub fn export_svg_with_resources_web_ready(
    doc: &Document,
    resources: &ResourceStore,
    canvas_width: f32,
    canvas_height: f32,
) -> String {
    export_svg_with_metadata_policy(
        ExportOptions::new(doc, resources, canvas_width, canvas_height),
        false,
    )
}
/// Export a web-ready SVG and validate resource references.
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub fn export_svg_with_resources_web_ready_checked(
    doc: &Document,
    resources: &ResourceStore,
    canvas_width: f32,
    canvas_height: f32,
) -> Result<String, SvgExportError> {
    export_svg_with_metadata_policy_checked(
        ExportOptions::new(doc, resources, canvas_width, canvas_height),
        false,
    )
}
/// Export options for metadata-aware SVG export.
#[derive(Clone, Copy)]
pub struct ExportOptions<'a> {
    /// Document to export.
    pub doc: &'a Document,
    /// Shared resources used by paint and defs export.
    pub resources: &'a ResourceStore,
    /// Canvas width in document units.
    pub canvas_width: f32,
    /// Canvas height in document units.
    pub canvas_height: f32,
    /// Optional raw metadata block content.
    pub metadata_block: Option<&'a str>,
}
impl<'a> ExportOptions<'a> {
    /// Create export options with no metadata block.
    #[must_use]
    pub const fn new(
        doc: &'a Document,
        resources: &'a ResourceStore,
        canvas_width: f32,
        canvas_height: f32,
    ) -> Self {
        Self {
            doc,
            resources,
            canvas_width,
            canvas_height,
            metadata_block: None,
        }
    }
    /// Attach metadata block content that should be preserved verbatim.
    #[must_use]
    pub const fn with_metadata_block(mut self, metadata_block: &'a str) -> Self {
        self.metadata_block = Some(metadata_block);
        self
    }
}
/// Export a document to an SVG string with metadata block preservation.
#[must_use]
pub fn export_svg_with_metadata(options: ExportOptions<'_>) -> String {
    export_svg_with_metadata_policy(options, true)
}
fn export_svg_with_metadata_policy(
    options: ExportOptions<'_>,
    preserve_gauss_metadata: bool,
) -> String {
    let mut out = String::new();
    write_svg_header(
        &mut out,
        options.canvas_width,
        options.canvas_height,
        preserve_gauss_metadata,
    );
    defs::write_defs(&mut out, options.resources);
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
/// Returns [`SvgExportError`] if a shape references a missing gradient/pattern.
pub fn export_svg_with_metadata_checked(
    options: ExportOptions<'_>,
) -> Result<String, SvgExportError> {
    export_svg_with_metadata_policy_checked(options, true)
}
fn export_svg_with_metadata_policy_checked(
    options: ExportOptions<'_>,
    preserve_gauss_metadata: bool,
) -> Result<String, SvgExportError> {
    validate_resource_references(options.doc, options.resources)?;
    Ok(export_svg_with_metadata_policy(
        options,
        preserve_gauss_metadata,
    ))
}
#[must_use]
pub(super) const fn opacity_from_alpha(alpha: u8) -> f32 {
    (alpha as f32) / 255.0
}
fn write_svg_header(
    out: &mut String,
    canvas_width: f32,
    canvas_height: f32,
    include_gauss_namespace: bool,
) {
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    if include_gauss_namespace {
        let gauss_namespace = gauss_namespace_declaration();
        write_fmt(
            out,
            format_args!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" {gauss_namespace} width="{canvas_width}" height="{canvas_height}" viewBox="0 0 {canvas_width} {canvas_height}">"#
            ),
        );
    } else {
        write_fmt(
            out,
            format_args!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{canvas_width}" height="{canvas_height}" viewBox="0 0 {canvas_width} {canvas_height}">"#
            ),
        );
    }
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
        write_shape_gauss_metadata(out, shape);
    }
    out.push_str(" />\n");
}
fn escape_attr_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
fn write_shape_gauss_metadata(out: &mut String, shape: &Shape) {
    if !shape.id.is_null() {
        let hex = shape_id_to_hex(shape.id);
        write_fmt(out, format_args!(r#" gauss:id="{hex}""#));
    }
    if let Some(name) = shape.name.as_deref() {
        let escaped_name = escape_attr_value(name);
        write_fmt(out, format_args!(r#" gauss:name="{escaped_name}""#));
    }
    if shape.locked {
        out.push_str(r#" gauss:locked="true""#);
    }
    if shape.hidden {
        out.push_str(r#" gauss:hidden="true""#);
    }
    write_opaque_gauss_attrs(out, &shape.gauss_metadata);
}
fn write_opaque_gauss_attrs(out: &mut String, metadata: &[crate::model::GaussAttribute]) {
    for attr in metadata {
        let escaped_value = escape_attr_value(&attr.value);
        write_fmt(
            out,
            format_args!(r#" gauss:{}="{escaped_value}""#, attr.name),
        );
    }
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
