//! SVG resource and paint parsing helpers.

use crate::model::{
    Gradient, GradientKind, GradientStop, LinearGradient, Paint, PatternResource, RadialGradient,
    ResourceStore, SymbolResource, Vec2,
};

use crate::svg::metadata::gauss_namespace_declaration;

use super::SvgImportError;
use super::resource_tag_attributes::collect_extra_attributes;
pub(super) use super::types::{AttributeName, SvgContent, TagName};

mod paint;
use paint::{
    parse_colour, parse_colour_with_resources, parse_offset, parse_opacity_to_alpha,
    parse_optional_f32,
};

/// Parse SVG resource definitions into a [`ResourceStore`].
///
/// This currently imports gradients, patterns, and symbols from supported SVG
/// subsets used by Gauss import/export round-trip flows.
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

/// Store each linear-gradient block with its parsed endpoints and stops.
///
/// Missing coordinates default to `(x1, y1) = (0, 0)` and `(x2, y2) = (1, 0)`;
/// for example, `<linearGradient/>` stores those endpoints and no stops.
/// Missing IDs become empty strings, and numeric or stop errors propagate.
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

/// Store each radial-gradient block with its parsed centre, radius, and stops.
///
/// Missing centre coordinates and radius default to `0.5`; a focal point equal
/// to the centre is omitted. For example, `<radialGradient/>` stores centre
/// `(0.5, 0.5)`, radius `0.5`, and no focal point. Missing IDs become empty
/// strings, and numeric or stop errors propagate.
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

/// Store pattern blocks with their inner body and non-ID attributes.
///
/// For example, a pattern with `id="p"` and `data-kind="tile"` stores the
/// body and `data-kind`; `id` is stored separately by the resource store.
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

/// Store symbol blocks with their body, view box, and remaining attributes.
///
/// For example, a symbol's `viewBox` is stored separately while attributes
/// such as `data-kind` remain attached to the resource.
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

/// Pass each matching resource block's fields to `insert_fn`.
///
/// The callback receives the ID (empty when absent), inner body, attributes
/// other than `excluded_attributes`, and the original block. For example,
/// excluding `id` passes it separately and leaves `data-kind` in the list.
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

/// Parse a paint attribute and optional opacity attribute into typed paint.
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

/// Extract all tags matching `tag_name` from `svg`.
pub(super) fn extract_single_tags(svg: SvgContent<'_>, tag_name: TagName<'_>) -> Vec<String> {
    extract_block_tags(svg, tag_name)
}

/// Extract drawable `<path>` tags while excluding resource-definition contexts.
pub(super) fn extract_shape_path_tags(svg: SvgContent<'_>) -> Vec<String> {
    extract_matching_tags(svg, |node| {
        node.tag_name().name() == "path" && !has_resource_ancestor(node)
    })
}

/// Read a named attribute value from the first element in `tag`.
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

/// Parse a gradient's stops into solid colours with resolved alpha values.
///
/// Missing offsets default to `0`, missing colours to opaque black, and
/// missing opacity to `255`. For example, `<stop/>` becomes an opaque black
/// stop at offset `0`; invalid numeric values and non-solid colours return an
/// import error.
fn parse_gradient_stops(block: SvgContent<'_>) -> Result<Vec<GradientStop>, SvgImportError> {
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

/// Extract full element blocks matching `tag_name` from `svg`.
pub(super) fn extract_block_tags(svg: SvgContent<'_>, tag_name: TagName<'_>) -> Vec<String> {
    extract_matching_tags(svg, |node| node.tag_name().name() == tag_name.as_str())
}

/// Return source slices for element nodes accepted by `is_match`.
///
/// Matches are collected in document order; for example, a predicate selecting
/// `path` returns each matching path's original XML slice. Parse failure yields
/// an empty vector.
fn extract_matching_tags(
    svg: SvgContent<'_>,
    is_match: impl Fn(roxmltree::Node<'_, '_>) -> bool,
) -> Vec<String> {
    with_parsed_document(svg.as_str(), |source, document| {
        document
            .root_element()
            .descendants()
            .filter(|node| node.is_element() && is_match(*node))
            .filter_map(|node| source.get(node.range()).map(ToOwned::to_owned))
            .collect()
    })
    .unwrap_or_default()
}

/// Return the source span between a block element's opening and closing tags.
///
/// For example, `<pattern><rect/></pattern>` yields `<rect/>`; an element with
/// no children, or a fragment that cannot be parsed, yields an empty string.
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

/// Check whether an element is nested beneath `defs`, `pattern`, or `symbol`.
///
/// The node itself is excluded, so a root-level `<path>` returns `false`, while
/// a `<path>` inside `<defs>` returns `true`.
fn has_resource_ancestor(node: roxmltree::Node<'_, '_>) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        ancestor.is_element() && matches!(ancestor.tag_name().name(), "defs" | "pattern" | "symbol")
    })
}

/// Parse a document or retry a fragment inside the canonical Gauss wrapper.
///
/// An unbound `gauss` prefix in a fragment can use the wrapper's declaration.
/// The callback receives the source used for the successful parse; `None` means
/// both the direct and wrapped parse failed.
fn with_parsed_document<T>(
    fragment: &str,
    map: impl FnOnce(&str, &roxmltree::Document<'_>) -> T,
) -> Option<T> {
    if let Ok(document) = roxmltree::Document::parse(fragment) {
        return Some(map(fragment, &document));
    }

    let ns = gauss_namespace_declaration();
    let wrapped = format!("<gauss-import-wrapper {ns}>{fragment}</gauss-import-wrapper>");
    let document = roxmltree::Document::parse(wrapped.as_str()).ok()?;
    Some(map(wrapped.as_str(), &document))
}
