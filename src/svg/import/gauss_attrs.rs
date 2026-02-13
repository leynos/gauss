//! Gauss-namespaced attribute extraction from SVG documents.
//!
//! Uses a full-document `roxmltree` parse for namespace-aware attribute
//! reading. The fragment-based `attribute_value()` in `resource_tags.rs`
//! loses namespace context, making it unsuitable for `gauss:*` attributes.

use crate::svg::metadata::GAUSS_METADATA_NAMESPACE;

/// Known Gauss attribute local names handled explicitly by the importer.
const KNOWN_GAUSS_ATTRS: &[&str] = &["id", "name", "locked", "hidden"];

/// Gauss-specific metadata extracted from a shape element.
#[derive(Default)]
pub(super) struct ShapeGaussMetadata {
    /// Value of `gauss:id`, if present.
    pub gauss_id: Option<String>,
    /// Value of `gauss:name`, if present.
    pub name: Option<String>,
    /// Whether `gauss:locked` is `"true"`.
    pub locked: bool,
    /// Whether `gauss:hidden` is `"true"`.
    pub hidden: bool,
    /// Unknown `gauss:*` attributes preserved for forward compatibility.
    pub opaque_attrs: Vec<(String, String)>,
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
pub(super) fn extract_metadata_block(svg: &str) -> Option<String> {
    let document = roxmltree::Document::parse(svg).ok()?;
    let root = document.root_element();

    let metadata_node = root
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "metadata")?;

    // Collect the raw source of all children within `<metadata>`.
    let first_child = metadata_node.first_child()?;
    let last_child = metadata_node.last_child().unwrap_or(first_child);
    let range = first_child.range().start..last_child.range().end;
    svg.get(range).map(ToOwned::to_owned)
}

fn extract_gauss_metadata_from_node(node: roxmltree::Node<'_, '_>) -> ShapeGaussMetadata {
    let mut meta = ShapeGaussMetadata::default();

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
                if !KNOWN_GAUSS_ATTRS.contains(&local_name) {
                    meta.opaque_attrs
                        .push((local_name.to_owned(), value.to_owned()));
                }
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
