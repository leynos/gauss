//! SVG export for the Gauss document model.
//!
//! The exporter is intentionally conservative: it emits `<path>` elements with
//! absolute commands (`M`, `L`, `C`, `Z`) and basic stroke/fill styling. This
//! keeps the output easy to inspect and easy to round-trip.

use std::fmt::{Arguments, Write as _};

use crate::model::{Document, Rgba, SegmentKind, format_hex_rgb};

/// Export a document to an SVG string.
///
/// The returned string is a complete SVG document including XML declaration and
/// root `<svg>` element with a `viewBox`.
///
/// `canvas_width` and `canvas_height` are treated as document units, which are
/// interpreted as pixels for the Phase 0 `PoC`.
#[must_use]
pub fn export_svg(doc: &Document, canvas_width: f32, canvas_height: f32) -> String {
    let mut out = String::new();
    write_svg_header(&mut out, canvas_width, canvas_height);
    for shape in &doc.shapes {
        write_shape_path(&mut out, shape);
    }
    out.push_str("</svg>\n");
    out
}

#[must_use]
fn opacity_from_alpha(alpha: u8) -> f32 {
    f32::from(alpha) / 255.0
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

fn write_shape_path(out: &mut String, shape: &crate::model::Shape) {
    let Some(path_data) = build_path_data(shape) else {
        return;
    };

    let (stroke_attr, stroke_opacity) = format_paint(shape.style.stroke);
    let (fill_attr, fill_opacity) = format_paint(shape.style.fill);

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

fn format_paint(paint: Option<Rgba>) -> (String, f32) {
    paint.map_or_else(
        || ("none".to_owned(), 1.0),
        |c| (format_hex_rgb(c), opacity_from_alpha(c.a)),
    )
}

fn write_optional_opacity(out: &mut String, attr: &str, paint: &str, opacity: f32) {
    if paint == "none" || opacity >= 1.0 {
        return;
    }

    write_fmt(out, format_args!(r#" {attr}="{opacity:.4}""#));
}

fn write_fmt(out: &mut String, args: Arguments<'_>) {
    if out.write_fmt(args).is_err() {
        // `String` implements `fmt::Write` infallibly, so ignore this case.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Anchor, PaintStyle, PathGeom, Shape, ShapeId, Vec2};
    use rstest::rstest;
    use uuid::Uuid;

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
            id: ShapeId::from(Uuid::from_u128(1)),
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

        let doc = Document {
            shapes: vec![shape],
        };
        let svg = export_svg(&doc, 10.0, 10.0);

        assert!(svg.contains(r#"d="M 1 2 L 3 4""#));
        assert!(svg.contains(r##"stroke="#000000""##));
        assert!(svg.contains(r#"stroke-width="1""#));
        assert!(svg.contains(r#"fill="none""#));
    }

    #[rstest]
    fn exports_opacity_when_alpha_is_not_opaque() {
        let shape = Shape {
            id: ShapeId::from(Uuid::from_u128(2)),
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

        let doc = Document {
            shapes: vec![shape],
        };
        let svg = export_svg(&doc, 10.0, 10.0);

        assert!(svg.contains(r#"stroke-opacity=""#));
        assert!(svg.contains(r#"fill-opacity=""#));
    }
}
