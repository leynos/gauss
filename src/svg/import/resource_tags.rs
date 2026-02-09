//! SVG resource and paint parsing helpers.

use crate::model::{
    Gradient, GradientKind, GradientStop, LinearGradient, Paint, PatternResource, RadialGradient,
    ResourceStore, SymbolResource, Vec2, parse_hex_rgb,
};

use super::SvgImportError;
use super::resource_tag_attributes::{collect_extra_attributes, opening_tag};
pub(super) use super::types::{AttributeName, SvgContent, TagName};

pub(super) fn parse_resources(
    svg: SvgContent<'_>,
    resources: &mut ResourceStore,
) -> Result<(), SvgImportError> {
    parse_linear_gradients(svg, resources)?;
    parse_radial_gradients(svg, resources)?;
    parse_patterns(svg, resources);
    parse_symbols(svg, resources);

    Ok(())
}

fn parse_linear_gradients(
    svg: SvgContent<'_>,
    resources: &mut ResourceStore,
) -> Result<(), SvgImportError> {
    for raw_block in extract_block_tags(svg, TagName::new("linearGradient")) {
        let block_content = SvgContent::new(raw_block.as_str());
        let id = attribute_value(block_content, AttributeName::new("id")).unwrap_or_default();
        let x1 = parse_optional_f32(
            attribute_value(block_content, AttributeName::new("x1")),
            0.0,
        )?;
        let y1 = parse_optional_f32(
            attribute_value(block_content, AttributeName::new("y1")),
            0.0,
        )?;
        let x2 = parse_optional_f32(
            attribute_value(block_content, AttributeName::new("x2")),
            1.0,
        )?;
        let y2 = parse_optional_f32(
            attribute_value(block_content, AttributeName::new("y2")),
            0.0,
        )?;
        let stops = parse_gradient_stops(block_content)?;
        let gradient = Gradient::new(
            id,
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(x1, y1),
                Vec2::new(x2, y2),
                stops,
            )),
        );
        let _gradient_id = resources.insert_gradient(gradient);
    }

    Ok(())
}

fn parse_radial_gradients(
    svg: SvgContent<'_>,
    resources: &mut ResourceStore,
) -> Result<(), SvgImportError> {
    for raw_block in extract_block_tags(svg, TagName::new("radialGradient")) {
        let block_content = SvgContent::new(raw_block.as_str());
        let id = attribute_value(block_content, AttributeName::new("id")).unwrap_or_default();
        let cx = parse_optional_f32(
            attribute_value(block_content, AttributeName::new("cx")),
            0.5,
        )?;
        let cy = parse_optional_f32(
            attribute_value(block_content, AttributeName::new("cy")),
            0.5,
        )?;
        let r = parse_optional_f32(attribute_value(block_content, AttributeName::new("r")), 0.5)?;
        let fx = parse_optional_f32(attribute_value(block_content, AttributeName::new("fx")), cx)?;
        let fy = parse_optional_f32(attribute_value(block_content, AttributeName::new("fy")), cy)?;
        let focal = if (fx - cx).abs() <= f32::EPSILON && (fy - cy).abs() <= f32::EPSILON {
            None
        } else {
            Some(Vec2::new(fx, fy))
        };
        let stops = parse_gradient_stops(block_content)?;
        let gradient = Gradient::new(
            id,
            GradientKind::Radial(RadialGradient::new(Vec2::new(cx, cy), r, focal, stops)),
        );
        let _gradient_id = resources.insert_gradient(gradient);
    }

    Ok(())
}

fn parse_patterns(svg: SvgContent<'_>, resources: &mut ResourceStore) {
    for raw_block in extract_block_tags(svg, TagName::new("pattern")) {
        let block_content = SvgContent::new(raw_block.as_str());
        let id = attribute_value(block_content, AttributeName::new("id")).unwrap_or_default();
        let body = inner_tag_body(block_content, TagName::new("pattern"));
        let opening_tag = opening_tag(block_content.as_str());
        let extra_attributes = opening_tag
            .map(|tag| collect_extra_attributes(tag, &["id"]))
            .unwrap_or_default();
        let _pattern_id = resources.insert_pattern(PatternResource::new_with_attributes(
            id,
            body,
            extra_attributes,
        ));
    }
}

fn parse_symbols(svg: SvgContent<'_>, resources: &mut ResourceStore) {
    for raw_block in extract_block_tags(svg, TagName::new("symbol")) {
        let block_content = SvgContent::new(raw_block.as_str());
        let id = attribute_value(block_content, AttributeName::new("id")).unwrap_or_default();
        let view_box = attribute_value(block_content, AttributeName::new("viewBox"));
        let body = inner_tag_body(block_content, TagName::new("symbol"));
        let opening_tag = opening_tag(block_content.as_str());
        let extra_attributes = opening_tag
            .map(|tag| collect_extra_attributes(tag, &["id", "viewBox"]))
            .unwrap_or_default();
        let _symbol_id = resources.insert_symbol(SymbolResource::new_with_attributes(
            id,
            view_box,
            body,
            extra_attributes,
        ));
    }
}

pub(super) fn parse_paint_with_opacity(
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

pub(super) fn extract_single_tags(svg: SvgContent<'_>, tag_name: TagName<'_>) -> Vec<String> {
    let mut tags = Vec::new();
    let mut remaining = svg.as_str();
    let tag_prefix = format!("<{}", tag_name.as_str());

    while let Some(start) = remaining.find(tag_prefix.as_str()) {
        let Some(after_start) = remaining.get(start..) else {
            break;
        };
        let Some(end) = after_start.find('>') else {
            break;
        };
        let Some(tag) = after_start.get(..=end) else {
            break;
        };
        tags.push(tag.to_owned());
        remaining = after_start.get((end + 1)..).unwrap_or_default();
    }

    tags
}

pub(super) fn attribute_value(tag: SvgContent<'_>, name: AttributeName<'_>) -> Option<String> {
    let tag_content = tag.as_str();
    for quote in ['"', '\''] {
        let needle = format!("{}={quote}", name.as_str());
        let Some(start) = tag_content.find(needle.as_str()) else {
            continue;
        };
        let Some(after) = tag_content.get((start + needle.len())..) else {
            continue;
        };
        let Some(end) = after.find(quote) else {
            continue;
        };
        if let Some(value) = after.get(..end) {
            return Some(value.to_owned());
        }
    }

    None
}

fn parse_gradient_stops(block: SvgContent<'_>) -> Result<Vec<GradientStop>, SvgImportError> {
    let mut stops = Vec::new();

    for raw_stop_tag in extract_single_tags(block, TagName::new("stop")) {
        let stop_tag_content = SvgContent::new(raw_stop_tag.as_str());
        let offset = attribute_value(stop_tag_content, AttributeName::new("offset"))
            .map(|value| parse_offset(value.as_str()))
            .transpose()?
            .unwrap_or(0.0);

        let colour = attribute_value(stop_tag_content, AttributeName::new("stop-color"))
            .ok_or(SvgImportError::InvalidColour)
            .and_then(|value| parse_colour(value.as_str()))?;

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

fn parse_optional_f32(value: Option<String>, default: f32) -> Result<f32, SvgImportError> {
    value.map_or(Ok(default), |raw| {
        raw.parse::<f32>()
            .map_err(|_| SvgImportError::InvalidNumber)
    })
}

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

fn extract_block_tags(svg: SvgContent<'_>, tag_name: TagName<'_>) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut remaining = svg.as_str();
    let open_prefix = format!("<{}", tag_name.as_str());
    let close_tag = format!("</{}>", tag_name.as_str());

    while let Some(start) = remaining.find(open_prefix.as_str()) {
        let Some(after_start) = remaining.get(start..) else {
            break;
        };
        let Some(open_end) = after_start.find('>') else {
            break;
        };

        let Some(open_tag) = after_start.get(..=open_end) else {
            break;
        };

        if open_tag.trim_end().ends_with("/>") {
            blocks.push(open_tag.to_owned());
            remaining = after_start.get((open_end + 1)..).unwrap_or_default();
            continue;
        }

        let Some(close_index_rel) = after_start.get((open_end + 1)..).and_then(|rest| {
            rest.find(close_tag.as_str())
                .map(|index| index + open_end + 1 + close_tag.len())
        }) else {
            break;
        };

        let Some(block) = after_start.get(..close_index_rel) else {
            break;
        };
        blocks.push(block.to_owned());
        remaining = after_start.get(close_index_rel..).unwrap_or_default();
    }

    blocks
}

fn inner_tag_body(block: SvgContent<'_>, tag_name: TagName<'_>) -> String {
    let block_content = block.as_str();
    let Some(open_end) = block_content.find('>') else {
        return String::new();
    };

    let Some(open_tag) = block_content.get(..=open_end) else {
        return String::new();
    };
    if open_tag.trim_end().ends_with("/>") {
        return String::new();
    }

    let close_tag = format!("</{}>", tag_name.as_str());
    let Some(close_start) = block_content.rfind(close_tag.as_str()) else {
        return String::new();
    };

    block_content
        .get((open_end + 1)..close_start)
        .map_or_else(String::new, ToOwned::to_owned)
}
