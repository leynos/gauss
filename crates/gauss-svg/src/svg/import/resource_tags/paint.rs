//! Paint, colour, and gradient-stop parsing for SVG import.

use super::{
    super::{
        SvgImportError,
        types::{AttributeName, SvgContent, TagName},
    },
    attribute_value,
    extract_single_tags,
};
use crate::model::{GradientStop, Paint, ResourceStore, parse_hex_rgb};

/// Parse a paint attribute and optional opacity attribute into typed paint.
pub(in crate::svg::import) fn parse_paint_with_opacity(
    tag: SvgContent<'_>,
    paint_attr: AttributeName<'_>,
    opacity_attr: AttributeName<'_>,
    resources: &ResourceStore,
) -> Result<Paint, SvgImportError> {
    let paint = match attribute_value(tag, paint_attr) {
        None => Paint::None,
        Some(value) => parse_colour_with_resources(value.as_str(), resources)?,
    };
    let alpha = match attribute_value(tag, opacity_attr) {
        None => 255,
        Some(value) => parse_opacity_to_alpha(value.as_str())?,
    };
    Ok(paint.with_opacity(alpha))
}

/// Parse all `<stop>` tags in a gradient body into typed gradient stops.
///
/// Offsets default to `0.0`, colours to opaque black, and opacity to full.
pub(super) fn parse_gradient_stops(
    block: SvgContent<'_>,
) -> Result<Vec<GradientStop>, SvgImportError> {
    let mut stops = Vec::new();

    for raw_stop_tag in extract_single_tags(block, TagName::new("stop")) {
        let stop_tag_content = SvgContent::new(raw_stop_tag.as_str());
        let offset = attribute_value(stop_tag_content, AttributeName::new("offset"))
            .map(|value| parse_offset(value.as_str()))
            .transpose()?
            .unwrap_or(0.0);

        let colour = attribute_value(stop_tag_content, AttributeName::new("stop-color"))
            .map_or_else(
                || Ok(Paint::Solid(crate::model::Rgba::new(0, 0, 0, 255))),
                |value| parse_colour(value.as_str()),
            )?;

        let alpha = attribute_value(stop_tag_content, AttributeName::new("stop-opacity"))
            .map(|value| parse_opacity_to_alpha(value.as_str()))
            .transpose()?
            .unwrap_or(255);

        let Paint::Solid(mut solid) = colour else {
            return Err(SvgImportError::InvalidColour);
        };
        solid.a = alpha;
        stops.push(GradientStop::new(offset, solid));
    }

    Ok(stops)
}

/// Parse `none`, a solid colour or a resource reference into `Paint`.
/// Known IDs resolve to paints; unknown IDs return `MissingReferencedResource`.
fn parse_colour_with_resources(
    value: &str,
    resources: &ResourceStore,
) -> Result<Paint, SvgImportError> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("none") {
        return Ok(Paint::None);
    }

    if let Some(svg_id) = parse_url_reference(trimmed) {
        if let Some(gradient_id) = resources.gradient_id_for_svg_id(svg_id.as_str()) {
            return Ok(Paint::gradient(gradient_id));
        }

        if let Some(pattern_id) = resources.pattern_id_for_svg_id(svg_id.as_str()) {
            return Ok(Paint::pattern(pattern_id));
        }

        return Err(SvgImportError::MissingReferencedResource(svg_id));
    }

    parse_colour(trimmed)
}

/// Match the literal `url(#` prefix and `)` suffix after trimming the value.
/// Blank inner text returns `None`; `url(# sky)` yields `Some(" sky")` unchanged.
fn parse_url_reference(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let prefix = "url(#";
    let without_prefix = trimmed.strip_prefix(prefix)?;
    let inner = without_prefix.strip_suffix(')')?;
    if inner.trim().is_empty() {
        None
    } else {
        Some(inner.to_owned())
    }
}

/// Parse a solid hexadecimal colour or the case-insensitive value `none`.
/// Hex colours become `Paint::Solid`; `none` becomes `Paint::None`.
fn parse_colour(value: &str) -> Result<Paint, SvgImportError> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("none") {
        return Ok(Paint::None);
    }

    if !trimmed.starts_with('#') {
        return Err(SvgImportError::InvalidColour);
    }

    parse_hex_rgb(trimmed)
        .map(Paint::Solid)
        .map_err(|_| SvgImportError::InvalidColour)
}

/// Convert normalized SVG opacity to an eight-bit alpha channel.
/// For example, `0.5` rounds to `128`; out-of-range values return `InvalidOpacity`.
fn parse_opacity_to_alpha(value: &str) -> Result<u8, SvgImportError> {
    let s = value.trim();
    if s.is_empty() {
        return Err(SvgImportError::InvalidOpacity);
    }

    let opacity = s
        .parse::<f32>()
        .map_err(|_| SvgImportError::InvalidOpacity)?;
    if !(0.0..=1.0).contains(&opacity) {
        return Err(SvgImportError::InvalidOpacity);
    }

    let scaled = (opacity * 255.0).round();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "opacity is validated to 0..=1 so the scaled value fits in u8"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "opacity is validated as non-negative before scaling"
    )]
    let alpha = scaled as u8;
    Ok(alpha)
}

/// Parse a gradient-stop offset as a number or percentage.
/// For example, `50%` becomes `0.5`; malformed text returns `InvalidNumber`.
fn parse_offset(value: &str) -> Result<f32, SvgImportError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SvgImportError::InvalidNumber);
    }

    if let Some(percent) = trimmed.strip_suffix('%') {
        let number = percent
            .parse::<f32>()
            .map_err(|_| SvgImportError::InvalidNumber)?;
        return Ok(number / 100.0);
    }

    trimmed
        .parse::<f32>()
        .map_err(|_| SvgImportError::InvalidNumber)
}
