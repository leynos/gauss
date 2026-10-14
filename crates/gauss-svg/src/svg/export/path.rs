//! SVG shape path and paint serialization.

use crate::model::{Paint, ResourceStore, SegmentKind, Shape, format_hex_rgb};

use super::{opacity_from_alpha, shape_metadata, write_fmt};

/// Write a shape as an SVG path with geometry and paint resources.
/// A shape without anchors adds no output.
pub(super) fn write_shape_path(
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
/// Build SVG path data from a shape’s anchors and segments.
/// An empty path returns `None`; a closed path ends with `Z`.
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
/// Format a paint value as SVG text and opacity.
/// Missing gradients or patterns become `none` with opacity one.
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
/// Append an opacity attribute unless the paint is `none` or nearly opaque.
/// For example, `none` or opacity near one adds no attribute.
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
