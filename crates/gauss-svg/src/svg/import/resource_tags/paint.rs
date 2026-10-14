//! Paint references, colours, opacity, and numeric SVG attributes.

use crate::model::{Paint, ResourceStore, parse_hex_rgb};

use super::SvgImportError;

/// Parse `none`, a solid colour or a resource reference into `Paint`.
/// Known IDs resolve to paints; unknown IDs return `MissingReferencedResource`.
pub(super) fn parse_colour_with_resources(
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
pub(super) fn parse_colour(value: &str) -> Result<Paint, SvgImportError> {
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
pub(super) fn parse_opacity_to_alpha(value: &str) -> Result<u8, SvgImportError> {
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

/// Parse an optional floating-point attribute or use its default.
/// Missing input returns `default`; malformed text returns `InvalidNumber`.
pub(super) fn parse_optional_f32(
    value: Option<String>,
    default: f32,
) -> Result<f32, SvgImportError> {
    value.map_or(Ok(default), |raw| {
        raw.parse::<f32>()
            .map_err(|_| SvgImportError::InvalidNumber)
    })
}

/// Parse a gradient-stop offset as a number or percentage.
/// For example, `50%` becomes `0.5`; malformed text returns `InvalidNumber`.
pub(super) fn parse_offset(value: &str) -> Result<f32, SvgImportError> {
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
