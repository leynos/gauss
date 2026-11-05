//! SVG resource and paint parsing helpers.

pub(super) use super::types::{AttributeName, SvgContent, TagName};
use super::{SvgImportError, resource_tag_attributes::collect_extra_attributes};
use crate::{
    model::{
        Gradient,
        GradientKind,
        LinearGradient,
        PatternResource,
        RadialGradient,
        ResourceStore,
        SymbolResource,
        Vec2,
    },
    svg::metadata::gauss_namespace_declaration,
};

mod paint;

use paint::parse_gradient_stops;
pub(super) use paint::parse_paint_with_opacity;

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

/// Import every `<linearGradient>` block into the resource store.
///
/// Missing coordinates fall back to the SVG defaults for a left-to-right
/// gradient.
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

/// Import every `<radialGradient>` block into the resource store.
///
/// A focal point coinciding with the centre is stored as `None`, matching the
/// exporter's omission of `fx`/`fy`.
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

/// Import every `<pattern>` block, preserving attributes other than `id`.
///
/// Preserved attributes are stored by local name, so a qualified attribute such
/// as `x:role` is kept as `role` and exported without its prefix. An extracted
/// block that relies on an ancestor's namespace declaration does not parse on
/// its own, so its extra attributes are dropped entirely.
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

/// Import every `<symbol>` block, treating `id` and `viewBox` as typed fields.
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

/// Walk blocks of one resource tag, extracting the shared fields and delegating
/// insertion to `insert_fn`.
///
/// `excluded_attributes` names attributes consumed as typed fields, so they are
/// not duplicated into the preserved extra attributes.
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

/// Parse an optional floating-point attribute or use its default.
/// Missing input returns `default`; malformed text returns `InvalidNumber`.
fn parse_optional_f32(value: Option<String>, default: f32) -> Result<f32, SvgImportError> {
    value.map_or(Ok(default), |raw| {
        raw.parse::<f32>()
            .map_err(|_| SvgImportError::InvalidNumber)
    })
}

/// Extract full element blocks matching `tag_name` from `svg`.
pub(super) fn extract_block_tags(svg: SvgContent<'_>, tag_name: TagName<'_>) -> Vec<String> {
    extract_matching_tags(svg, |node| node.tag_name().name() == tag_name.as_str())
}

/// Collect the source text of every element accepted by `is_match`.
///
/// Returns an empty vector when the fragment cannot be parsed.
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

/// Return the raw inner XML of a block, excluding its own tags.
///
/// An empty or unparsable block yields an empty string.
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

/// Report whether a node sits inside a resource container (`defs`, `pattern`,
/// or `symbol`), where shapes are definitions rather than drawable output.
fn has_resource_ancestor(node: roxmltree::Node<'_, '_>) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        ancestor.is_element() && matches!(ancestor.tag_name().name(), "defs" | "pattern" | "symbol")
    })
}

/// Parse a fragment and pass it, with its source text, to `map`.
///
/// Fragments that are not well-formed on their own are retried inside a
/// namespace-declaring wrapper, and `map` receives the wrapped source so byte
/// ranges still line up.
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
