//! SVG export for the Gauss document model.
//!
//! The exporter is intentionally conservative: it emits `<path>` elements with
//! absolute commands (`M`, `L`, `C`, `Z`) and writes resource definitions from
//! `ResourceStore` into `<defs>`.

#![expect(
    clippy::float_arithmetic,
    reason = "SVG export needs floating-point conversions for geometry"
)]

use std::fmt::{Arguments, Write as _};

use crate::model::{
    Document, GradientId, GradientKind, Paint, PatternId, PatternResource, ResourceStore,
    SegmentKind, SymbolResource, format_hex_rgb,
};

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

/// Export a document to an SVG string.
///
/// This compatibility helper exports without shared resources.
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
    // Compatibility path keeps lossy behaviour for callers that do not yet
    // handle export validation errors.
    let mut out = String::new();
    write_svg_header(&mut out, canvas_width, canvas_height);
    write_defs(&mut out, resources);
    for shape in doc.iter_in_draw_order() {
        write_shape_path(&mut out, resources, shape);
    }
    out.push_str("</svg>\n");
    out
}

/// Export a document to an SVG string with explicit shared resources.
///
/// This variant validates paint references first and returns an error when
/// a shape refers to a missing resource.
///
/// # Errors
///
/// Returns [`SvgExportError`] if any gradient/pattern paint reference cannot
/// be resolved in `resources`.
pub fn export_svg_with_resources_checked(
    doc: &Document,
    resources: &ResourceStore,
    canvas_width: f32,
    canvas_height: f32,
) -> Result<String, SvgExportError> {
    validate_resource_references(doc, resources)?;
    Ok(export_svg_with_resources(
        doc,
        resources,
        canvas_width,
        canvas_height,
    ))
}

#[must_use]
const fn opacity_from_alpha(alpha: u8) -> f32 {
    (alpha as f32) / 255.0
}

fn write_svg_header(out: &mut String, canvas_width: f32, canvas_height: f32) {
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');

    write_fmt(
        out,
        format_args!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{canvas_width}" height="{canvas_height}" viewBox="0 0 {canvas_width} {canvas_height}">"#
        ),
    );
    out.push('\n');
}

fn write_defs(out: &mut String, resources: &ResourceStore) {
    if resources.is_empty() {
        return;
    }

    out.push_str("<defs>\n");

    for (_id, gradient) in resources.gradients() {
        write_gradient(out, gradient);
    }

    for (_id, pattern) in resources.patterns() {
        write_pattern(out, pattern);
    }

    for (_id, symbol) in resources.symbols() {
        write_symbol(out, symbol);
    }

    out.push_str("</defs>\n");
}

fn write_gradient(out: &mut String, gradient: &crate::model::Gradient) {
    match &gradient.kind {
        GradientKind::Linear(data) => write_linear_gradient(out, gradient.svg_id.as_str(), data),
        GradientKind::Radial(data) => write_radial_gradient(out, gradient.svg_id.as_str(), data),
    }
}

fn write_linear_gradient(out: &mut String, svg_id: &str, data: &crate::model::LinearGradient) {
    write_fmt(
        out,
        format_args!(
            r#"<linearGradient id="{}" x1="{}" y1="{}" x2="{}" y2="{}">"#,
            svg_id, data.start.x, data.start.y, data.end.x, data.end.y
        ),
    );
    out.push('\n');

    for stop in &data.stops {
        write_gradient_stop(out, *stop);
    }

    out.push_str("</linearGradient>\n");
}

fn write_radial_gradient(out: &mut String, svg_id: &str, data: &crate::model::RadialGradient) {
    write_fmt(
        out,
        format_args!(
            r#"<radialGradient id="{}" cx="{}" cy="{}" r="{}""#,
            svg_id, data.centre.x, data.centre.y, data.radius
        ),
    );

    if let Some(focal) = data.focal {
        write_fmt(out, format_args!(r#" fx="{}" fy="{}""#, focal.x, focal.y));
    }

    out.push_str(">\n");

    for stop in &data.stops {
        write_gradient_stop(out, *stop);
    }

    out.push_str("</radialGradient>\n");
}

fn write_gradient_stop(out: &mut String, stop: crate::model::GradientStop) {
    let stop_colour = format_hex_rgb(stop.colour);
    write_fmt(
        out,
        format_args!(
            r#"<stop offset="{}" stop-color="{}" stop-opacity="{:.4}" />"#,
            stop.offset,
            stop_colour,
            opacity_from_alpha(stop.colour.a)
        ),
    );
    out.push('\n');
}

fn write_pattern(out: &mut String, pattern: &PatternResource) {
    write_fmt(out, format_args!(r#"<pattern id="{}""#, pattern.svg_id));
    write_extra_attributes(out, &pattern.extra_attributes);
    out.push('>');
    if !pattern.body.is_empty() {
        out.push_str(pattern.body.as_str());
    }
    out.push_str("</pattern>\n");
}

fn write_symbol(out: &mut String, symbol: &SymbolResource) {
    write_fmt(out, format_args!(r#"<symbol id="{}""#, symbol.svg_id));
    if let Some(view_box) = symbol.view_box.as_deref() {
        write_fmt(out, format_args!(r#" viewBox="{view_box}""#));
    }
    write_extra_attributes(out, &symbol.extra_attributes);
    out.push('>');
    if !symbol.body.is_empty() {
        out.push_str(symbol.body.as_str());
    }
    out.push_str("</symbol>\n");
}

fn write_extra_attributes(out: &mut String, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        write_fmt(out, format_args!(r#" {name}="{value}""#));
    }
}

fn write_shape_path(out: &mut String, resources: &ResourceStore, shape: &crate::model::Shape) {
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

    out.push_str(" />\n");
}

fn build_path_data(shape: &crate::model::Shape) -> Option<String> {
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

fn write_fmt(out: &mut String, args: Arguments<'_>) {
    if out.write_fmt(args).is_err() {
        // `String` implements `fmt::Write` without failing, so this is
        // unreachable in practice.
    }
}

#[cfg(test)]
mod tests {
    //! Tests for SVG export output.

    use super::*;
    use crate::model::{
        Anchor, Gradient, GradientKind, GradientStop, LinearGradient, PaintStyle, PathGeom,
        PatternResource, Rgba, Shape, SymbolResource, Vec2,
    };
    use crate::test_helpers::shape_id_from_seed as shape_id;
    use rstest::rstest;

    #[rstest]
    fn exports_empty_document_with_valid_svg_root() {
        let doc = Document::new();
        let svg = export_svg(&doc, 100.0, 50.0);
        assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
        assert!(svg.contains(r#"viewBox="0 0 100 50""#));
    }

    #[rstest]
    fn exports_simple_line_path() {
        let shape = Shape {
            id: shape_id(1),
            z: 0,
            style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None),
            path: PathGeom {
                anchors: vec![
                    Anchor::new(Vec2::new(1.0, 2.0)),
                    Anchor::new(Vec2::new(3.0, 4.0)),
                ],
                segments: vec![SegmentKind::Line],
                closed: false,
                closing_segment: SegmentKind::Line,
            },
        };

        let mut doc = Document::new();
        doc.append_shape(shape);
        let svg = export_svg(&doc, 10.0, 10.0);

        assert!(svg.contains(r#"d="M 1 2 L 3 4""#));
        assert!(svg.contains(r##"stroke="#000000""##));
        assert!(svg.contains(r#"stroke-width="1""#));
        assert!(svg.contains(r#"fill="none""#));
    }

    #[rstest]
    fn exports_opacity_when_alpha_is_not_opaque() {
        let shape = Shape {
            id: shape_id(2),
            z: 0,
            style: PaintStyle::new(
                Some(Rgba::new(255, 0, 0, 128)),
                2.0,
                Some(Rgba::new(0, 0, 0, 64)),
            ),
            path: PathGeom {
                anchors: vec![
                    Anchor::new(Vec2::new(0.0, 0.0)),
                    Anchor::new(Vec2::new(1.0, 1.0)),
                ],
                segments: vec![SegmentKind::Line],
                closed: true,
                closing_segment: SegmentKind::Line,
            },
        };

        let mut doc = Document::new();
        doc.append_shape(shape);
        let svg = export_svg(&doc, 10.0, 10.0);

        assert!(svg.contains(r#"stroke-opacity=""#));
        assert!(svg.contains(r#"fill-opacity=""#));
    }

    #[rstest]
    fn exports_gradient_and_pattern_defs_and_references() {
        let mut resources = ResourceStore::new();
        let gradient_id = resources.insert_gradient(Gradient::new(
            "sunset",
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                vec![
                    GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                    GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
                ],
            )),
        ));
        let pattern_id = resources.insert_pattern(PatternResource::new("dots", "<circle />"));

        let shape = Shape {
            id: shape_id(3),
            z: 0,
            style: PaintStyle::new_with_paint(
                Paint::gradient(gradient_id),
                1.5,
                Paint::pattern(pattern_id),
            ),
            path: PathGeom {
                anchors: vec![
                    Anchor::new(Vec2::new(0.0, 0.0)),
                    Anchor::new(Vec2::new(5.0, 0.0)),
                    Anchor::new(Vec2::new(5.0, 5.0)),
                ],
                segments: vec![SegmentKind::Line, SegmentKind::Line],
                closed: true,
                closing_segment: SegmentKind::Line,
            },
        };

        let mut doc = Document::new();
        doc.append_shape(shape);
        let svg = export_svg_with_resources(&doc, &resources, 10.0, 10.0);

        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<linearGradient id=\"sunset\""));
        assert!(svg.contains("<pattern id=\"dots\""));
        assert!(svg.contains("stroke=\"url(#sunset)\""));
        assert!(svg.contains("fill=\"url(#dots)\""));
    }

    #[rstest]
    fn exports_paint_server_opacity_attributes() {
        let mut resources = ResourceStore::new();
        let gradient_id = resources.insert_gradient(Gradient::new(
            "sunset",
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                vec![
                    GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                    GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
                ],
            )),
        ));
        let pattern_id = resources.insert_pattern(PatternResource::new("dots", "<circle />"));

        let shape = Shape {
            id: shape_id(30),
            z: 0,
            style: PaintStyle::new_with_paint(
                Paint::gradient(gradient_id).with_opacity(128),
                1.0,
                Paint::pattern(pattern_id).with_opacity(64),
            ),
            path: PathGeom {
                anchors: vec![
                    Anchor::new(Vec2::new(0.0, 0.0)),
                    Anchor::new(Vec2::new(5.0, 0.0)),
                    Anchor::new(Vec2::new(5.0, 5.0)),
                ],
                segments: vec![SegmentKind::Line, SegmentKind::Line],
                closed: true,
                closing_segment: SegmentKind::Line,
            },
        };

        let mut doc = Document::new();
        doc.append_shape(shape);
        let svg = export_svg_with_resources(&doc, &resources, 10.0, 10.0);

        assert!(svg.contains("stroke=\"url(#sunset)\""));
        assert!(svg.contains("fill=\"url(#dots)\""));
        assert!(svg.contains("stroke-opacity=\"0.5020\""));
        assert!(svg.contains("fill-opacity=\"0.2510\""));
    }

    #[rstest]
    fn exports_pattern_and_symbol_extra_attributes() {
        let mut resources = ResourceStore::new();
        let _pattern_id = resources.insert_pattern(PatternResource::new_with_attributes(
            "dots",
            "<circle />",
            vec![
                ("patternUnits".to_owned(), "userSpaceOnUse".to_owned()),
                ("patternTransform".to_owned(), "scale(2)".to_owned()),
            ],
        ));
        let _symbol_id = resources.insert_symbol(SymbolResource::new_with_attributes(
            "badge",
            Some("0 0 10 10".to_owned()),
            "<rect width=\"10\" height=\"10\" />",
            vec![("preserveAspectRatio".to_owned(), "xMidYMid".to_owned())],
        ));

        let svg = export_svg_with_resources(&Document::new(), &resources, 10.0, 10.0);
        assert!(svg.contains(
            "<pattern id=\"dots\" patternUnits=\"userSpaceOnUse\" patternTransform=\"scale(2)\">"
        ));
        assert!(svg.contains(
            "<symbol id=\"badge\" viewBox=\"0 0 10 10\" preserveAspectRatio=\"xMidYMid\">"
        ));
    }

    #[rstest]
    fn checked_export_reports_missing_resource_references() {
        let mut resources = ResourceStore::new();
        let dangling_gradient = resources.insert_gradient(Gradient::new(
            "dangling",
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                vec![
                    GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                    GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
                ],
            )),
        ));
        let _removed = resources.remove_gradient(dangling_gradient);

        let shape = Shape {
            id: shape_id(31),
            z: 0,
            style: PaintStyle::new_with_paint(Paint::gradient(dangling_gradient), 1.0, Paint::None),
            path: PathGeom {
                anchors: vec![
                    Anchor::new(Vec2::new(0.0, 0.0)),
                    Anchor::new(Vec2::new(5.0, 0.0)),
                    Anchor::new(Vec2::new(5.0, 5.0)),
                ],
                segments: vec![SegmentKind::Line, SegmentKind::Line],
                closed: true,
                closing_segment: SegmentKind::Line,
            },
        };

        let mut doc = Document::new();
        doc.append_shape(shape);

        let exported = export_svg_with_resources_checked(&doc, &resources, 10.0, 10.0);
        assert_eq!(
            exported,
            Err(SvgExportError::MissingGradientReference(dangling_gradient))
        );
    }
}
