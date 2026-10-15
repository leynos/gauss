//! SVG export for the Gauss document model.
#![expect(
    clippy::float_arithmetic,
    reason = "SVG export needs floating-point conversions for geometry"
)]
mod defs;
mod path;
mod shape_metadata;
use crate::model::{Document, GradientId, Paint, PatternId, ResourceStore};
use crate::svg::metadata::gauss_namespace_declaration;
use path::write_shape_path;
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
    /// Return whether this mode preserves Gauss metadata and namespace output.
    ///
    /// Example: `GaussWithMetadata` returns `true`; `WebReady` returns `false`.
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
/// Render the header, definitions, optional metadata, draw-order shapes, and
/// closing root tag.
///
/// Resource validation belongs to the checked wrapper, not this renderer.
/// Example: Web-ready output omits the Gauss namespace and metadata block.
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
/// Convert alpha from `0`–`255` to SVG opacity on the `0.0`–`1.0` scale.
///
/// Example: `255` produces `1.0`; `0` produces `0.0`.
#[must_use]
pub(super) const fn opacity_from_alpha(alpha: u8) -> f32 {
    (alpha as f32) / 255.0
}
/// Write the XML declaration and SVG root with canvas dimensions.
///
/// The canonical Gauss namespace is included only when requested.
/// Example: `false` omits `xmlns:gauss` from the root element.
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
/// Wrap present metadata content; `None` writes nothing.
///
/// Content is copied as supplied, with a newline added when needed.
/// Example: `"..."` becomes `<metadata>...\n</metadata>\n`.
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
/// Check both paints on each shape in draw order, stopping at the first
/// missing reference.
///
/// Missing gradients and patterns return their corresponding error; solid and
/// no-paint values pass.
/// Example: A missing fill pattern returns `MissingPatternReference`.
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
/// Check a gradient or pattern paint against the resource store.
///
/// Solid and no-paint values pass without a lookup.
/// Example: A missing pattern ID returns `MissingPatternReference`.
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
/// Append formatted arguments to the SVG output string.
///
/// String writes are infallible in practice, so this helper discards the
/// result.
/// Example: Formatting `width="12"` appends that attribute text.
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
