//! Helpers for parsing attribute lists from SVG opening tags.

/// Collect element attributes except for excluded names.
///
/// This helper is used for resource tags where selected attributes are parsed
/// into typed fields and all remaining attributes must be preserved for
/// round-trip export.
pub(super) fn collect_extra_attributes(
    tag: &str,
    excluded_names: &[&str],
) -> Vec<(String, String)> {
    with_first_element(tag, |element| {
        element
            .attributes()
            .filter(|attribute| !excluded_names.contains(&attribute.name()))
            .map(|attribute| (attribute.name().to_owned(), attribute.value().to_owned()))
            .collect()
    })
    .unwrap_or_default()
}

/// Apply `map` to the first element available in an XML fragment.
///
/// A complete `<pattern/>` is mapped directly. A fragment such as
/// `<path/> <circle/>` is retried in a wrapper and maps only its first child;
/// failure to parse either form, or finding no child element, returns `None`.
fn with_first_element<T>(
    fragment: &str,
    map: impl FnOnce(roxmltree::Node<'_, '_>) -> T,
) -> Option<T> {
    if let Ok(document) = roxmltree::Document::parse(fragment) {
        return Some(map(document.root_element()));
    }

    let wrapped = format!("<gauss-import-wrapper>{fragment}</gauss-import-wrapper>");
    let document = roxmltree::Document::parse(wrapped.as_str()).ok()?;
    let root = document.root_element();
    let element = if root.tag_name().name() == "gauss-import-wrapper" {
        root.children().find(roxmltree::Node::is_element)?
    } else {
        root
    };
    Some(map(element))
}
