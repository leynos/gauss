//! SVG `<defs>` block writing for resource definitions.
//!
//! Extracted from the main export module to keep file sizes manageable.

use crate::model::{GradientKind, PatternResource, ResourceStore, SymbolResource, format_hex_rgb};

use super::{opacity_from_alpha, write_fmt};

/// Write the `<defs>` block for shared resources.
pub(super) fn write_defs(out: &mut String, resources: &ResourceStore) {
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
