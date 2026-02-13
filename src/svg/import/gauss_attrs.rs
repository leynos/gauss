//! Gauss-namespaced attribute extraction from SVG documents.
//!
//! Uses a full-document `roxmltree` parse for namespace-aware attribute
//! reading. The fragment-based `attribute_value()` in `resource_tags.rs`
//! loses namespace context, making it unsuitable for `gauss:*` attributes.

use std::ops::Range;

use crate::model::GaussAttribute;
use crate::svg::metadata::GAUSS_METADATA_NAMESPACE;

/// Gauss-specific metadata extracted from a shape element.
pub(super) struct ShapeGaussMetadata {
    /// Byte range of the `<path>` node in the original SVG, for alignment with
    /// the tag-based extraction in `resource_tags`.
    pub byte_range: Range<usize>,
    /// Value of `gauss:id`, if present.
    pub gauss_id: Option<String>,
    /// Value of `gauss:name`, if present.
    pub name: Option<String>,
    /// Whether `gauss:locked` is `"true"`.
    pub locked: bool,
    /// Whether `gauss:hidden` is `"true"`.
    pub hidden: bool,
    /// Unknown `gauss:*` attributes preserved for forward compatibility.
    pub opaque_attrs: Vec<GaussAttribute>,
}

/// Extract Gauss metadata from all drawable path elements.
///
/// Iterates `<path>` elements not inside `<defs>`, `<pattern>`, or
/// `<symbol>` and reads `gauss:*` attributes from the Gauss namespace.
pub(super) fn extract_shape_gauss_metadata(svg: &str) -> Vec<ShapeGaussMetadata> {
    let Ok(document) = roxmltree::Document::parse(svg) else {
        return Vec::new();
    };

    document
        .root_element()
        .descendants()
        .filter(|node| {
            node.is_element() && node.tag_name().name() == "path" && !has_resource_ancestor(*node)
        })
        .map(|node| extract_gauss_metadata_from_node(node))
        .collect()
}

/// Extract raw inner content of the first `<metadata>` element.
///
/// Returns:
/// - `None` if there is no `<metadata>` element at all.
/// - `Some(String::new())` if `<metadata>` exists but is empty.
/// - `Some(contents)` with the raw inner XML if it has children.
pub(super) fn extract_metadata_block(svg: &str) -> Option<String> {
    let document = roxmltree::Document::parse(svg).ok()?;
    let root = document.root_element();

    let metadata_node = root
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "metadata")?;

    // If `<metadata>` is present but has no children, preserve it as an empty
    // block.
    let Some(first_child) = metadata_node.first_child() else {
        return Some(String::new());
    };

    // Collect the raw source of all children within `<metadata>`.
    let last_child = metadata_node.last_child().unwrap_or(first_child);
    let range = first_child.range().start..last_child.range().end;
    svg.get(range).map(ToOwned::to_owned)
}

fn extract_gauss_metadata_from_node(node: roxmltree::Node<'_, '_>) -> ShapeGaussMetadata {
    let mut meta = ShapeGaussMetadata {
        byte_range: node.range(),
        gauss_id: None,
        name: None,
        locked: false,
        hidden: false,
        opaque_attrs: Vec::new(),
    };

    for attr in node.attributes() {
        if attr.namespace() != Some(GAUSS_METADATA_NAMESPACE) {
            continue;
        }

        let local_name = attr.name();
        let value = attr.value();

        match local_name {
            "id" => meta.gauss_id = Some(value.to_owned()),
            "name" => meta.name = Some(value.to_owned()),
            "locked" => meta.locked = value == "true",
            "hidden" => meta.hidden = value == "true",
            _ => {
                meta.opaque_attrs
                    .push(GaussAttribute::new(local_name, value));
            }
        }
    }

    meta
}

fn has_resource_ancestor(node: roxmltree::Node<'_, '_>) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        ancestor.is_element() && matches!(ancestor.tag_name().name(), "defs" | "pattern" | "symbol")
    })
}
