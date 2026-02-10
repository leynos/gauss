//! SVG resource and paint parsing helpers.

use crate::model::{
    Gradient, GradientKind, GradientStop, LinearGradient, Paint, PatternResource, RadialGradient,
    ResourceStore, SymbolResource, Vec2, parse_hex_rgb,
};

use super::SvgImportError;
use super::resource_tag_attributes::collect_extra_attributes;
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
    parse_resource_blocks(
        svg,
        TagName::new("pattern"),
        &["id"],
        |id, body, extra_attributes, _block_content| {
            let _pattern_id = resources.insert_pattern(PatternResource::new_with_attributes(
                id,
                body,
                extra_attributes,
            ));
        },
    );
}

fn parse_symbols(svg: SvgContent<'_>, resources: &mut ResourceStore) {
    parse_resource_blocks(
        svg,
        TagName::new("symbol"),
        &["id", "viewBox"],
        |id, body, extra_attributes, block_content| {
            let view_box = attribute_value(block_content, AttributeName::new("viewBox"));
            let _symbol_id = resources.insert_symbol(SymbolResource::new_with_attributes(
                id,
                view_box,
                body,
                extra_attributes,
            ));
        },
    );
}

fn parse_resource_blocks<F>(
    svg: SvgContent<'_>,
    tag_name: TagName<'_>,
    excluded_attributes: &[&str],
    mut insert_fn: F,
) where
    F: for<'a> FnMut(String, String, Vec<(String, String)>, SvgContent<'a>),
{
    for raw_block in extract_block_tags(svg, tag_name) {
        let block_content = SvgContent::new(raw_block.as_str());
        let id = attribute_value(block_content, AttributeName::new("id")).unwrap_or_default();
        let body = inner_tag_body(block_content);
        let extra_attributes =
            collect_extra_attributes(block_content.as_str(), excluded_attributes);
        insert_fn(id, body, extra_attributes, block_content);
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
    extract_block_tags(svg, tag_name)
}

pub(super) fn attribute_value(tag: SvgContent<'_>, name: AttributeName<'_>) -> Option<String> {
    with_parsed_document(tag.as_str(), |_source, document| {
        let root = document.root_element();
        let element = if root.tag_name().name() == "gauss-import-wrapper" {
            root.children().find(roxmltree::Node::is_element)?
        } else {
            root
        };
        element.attribute(name.as_str()).map(ToOwned::to_owned)
    })
    .flatten()
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

pub(super) fn extract_block_tags(svg: SvgContent<'_>, tag_name: TagName<'_>) -> Vec<String> {
    with_parsed_document(svg.as_str(), |source, document| {
        document
            .root_element()
            .descendants()
            .filter(|node| node.is_element() && node.tag_name().name() == tag_name.as_str())
            .filter_map(|node| source.get(node.range()).map(ToOwned::to_owned))
            .collect()
    })
    .unwrap_or_default()
}

fn inner_tag_body(block: SvgContent<'_>) -> String {
    with_parsed_document(block.as_str(), |source, document| {
        let root = document.root_element();
        let element = if root.tag_name().name() == "gauss-import-wrapper" {
            root.children().find(roxmltree::Node::is_element)?
        } else {
            root
        };
        let mut children = element.children();
        let first_child = children.next()?;
        let last_child = children.next_back().unwrap_or(first_child);

        source
            .get(first_child.range().start..last_child.range().end)
            .map(ToOwned::to_owned)
    })
    .flatten()
    .unwrap_or_default()
}

fn with_parsed_document<T>(
    fragment: &str,
    map: impl FnOnce(&str, &roxmltree::Document<'_>) -> T,
) -> Option<T> {
    if let Ok(document) = roxmltree::Document::parse(fragment) {
        return Some(map(fragment, &document));
    }

    let wrapped = format!("<gauss-import-wrapper>{fragment}</gauss-import-wrapper>");
    let document = roxmltree::Document::parse(wrapped.as_str()).ok()?;
    Some(map(wrapped.as_str(), &document))
}
