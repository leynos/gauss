//! Gauss SVG metadata namespace policy.
//!
//! This module defines the canonical Gauss metadata namespace and validates
//! namespace usage in imported SVG documents.

use std::{error::Error, fmt, sync::LazyLock};

use slotmap::{Key, KeyData};

use crate::model::ShapeId;

/// Canonical namespace prefix for Gauss metadata.
pub const GAUSS_METADATA_PREFIX: &str = "gauss";
/// Canonical namespace URI for Gauss metadata.
pub const GAUSS_METADATA_NAMESPACE: &str = "https://gauss.dev/ns/metadata/1";

static GAUSS_NAMESPACE_DECLARATION: LazyLock<String> =
    LazyLock::new(|| format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#));

/// Errors produced while validating namespace policy in imported SVG content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamespacePolicyError {
    /// SVG could not be parsed as XML.
    InvalidXml,
    /// `gauss` prefix is bound to an unexpected namespace URI.
    InvalidGaussNamespaceBinding(String),
    /// Gauss metadata namespace is used without canonical `xmlns:gauss`
    /// declaration.
    MissingGaussNamespaceDeclaration,
}

impl fmt::Display for NamespacePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml => write!(f, "invalid SVG XML"),
            Self::InvalidGaussNamespaceBinding(found) => {
                write!(
                    f,
                    "gauss namespace prefix points to unexpected URI '{found}'"
                )
            }
            Self::MissingGaussNamespaceDeclaration => {
                write!(
                    f,
                    "Gauss metadata namespace usage requires canonical xmlns:gauss declaration"
                )
            }
        }
    }
}

impl Error for NamespacePolicyError {}

/// Returns canonical `xmlns:gauss="..."` declaration.
#[must_use]
pub fn gauss_namespace_declaration() -> &'static str {
    GAUSS_NAMESPACE_DECLARATION.as_str()
}

/// Validate Gauss metadata namespace usage for imported SVG content.
///
/// Rules:
///
/// - If `gauss` prefix is declared, it must point to the canonical URI.
/// - If the Gauss namespace URI is used by any element or attribute,
///   `xmlns:gauss="..."` must be declared on the SVG root.
///
/// # Errors
///
/// Returns [`NamespacePolicyError::InvalidXml`] when the SVG cannot be parsed.
/// Returns [`NamespacePolicyError::InvalidGaussNamespaceBinding`] when
/// `xmlns:gauss` points to a non-canonical URI.
/// Returns [`NamespacePolicyError::MissingGaussNamespaceDeclaration`] when
/// Gauss namespace URI content exists without canonical `xmlns:gauss`
/// declaration.
pub fn validate_namespace_policy(svg: &str) -> Result<(), NamespacePolicyError> {
    let document = roxmltree::Document::parse(svg).map_err(|_| NamespacePolicyError::InvalidXml)?;
    let root = document.root_element();

    if let Some(non_canonical) = first_non_canonical_gauss_binding(&document) {
        return Err(NamespacePolicyError::InvalidGaussNamespaceBinding(
            non_canonical,
        ));
    }

    let declared_uri = root.lookup_namespace_uri(Some(GAUSS_METADATA_PREFIX));
    let uses_gauss_namespace = document.descendants().any(node_uses_gauss_namespace);
    if uses_gauss_namespace && declared_uri != Some(GAUSS_METADATA_NAMESPACE) {
        return Err(NamespacePolicyError::MissingGaussNamespaceDeclaration);
    }

    Ok(())
}

fn node_uses_gauss_namespace(node: roxmltree::Node<'_, '_>) -> bool {
    node.is_element()
        && (node.tag_name().namespace() == Some(GAUSS_METADATA_NAMESPACE)
            || node
                .attributes()
                .any(|attribute| attribute.namespace() == Some(GAUSS_METADATA_NAMESPACE)))
}

fn first_non_canonical_gauss_binding(document: &roxmltree::Document<'_>) -> Option<String> {
    document
        .descendants()
        .filter(roxmltree::Node::is_element)
        .find_map(|node| {
            node.lookup_namespace_uri(Some(GAUSS_METADATA_PREFIX))
                .filter(|uri| *uri != GAUSS_METADATA_NAMESPACE)
                .map(ToOwned::to_owned)
        })
}

/// Encode a [`ShapeId`] as a 16-digit zero-padded lowercase hexadecimal string.
///
/// The encoding uses [`KeyData::as_ffi()`] for a stable, round-trippable
/// representation that is proven by AccessKit integration.
///
/// # Examples
///
/// ```rust
/// use gauss::svg::metadata::shape_id_to_hex;
/// use gauss::model::ShapeId;
///
/// let hex = shape_id_to_hex(ShapeId::default());
/// assert_eq!(hex.len(), 16);
/// ```
#[must_use]
pub fn shape_id_to_hex(id: ShapeId) -> String {
    format!("{:016x}", id.data().as_ffi())
}

/// Decode a [`ShapeId`] from a hexadecimal string.
///
/// Returns `None` for invalid hex, null keys, or strings that do not
/// represent a valid `KeyData` round-trip.
///
/// # Examples
///
/// ```rust
/// use gauss::svg::metadata::{shape_id_from_hex, shape_id_to_hex};
/// use gauss::model::ShapeId;
/// use slotmap::KeyData;
///
/// let id = KeyData::from_ffi(0x0000_0001_0000_0000).into();
/// let hex = shape_id_to_hex(id);
/// assert_eq!(shape_id_from_hex(&hex), Some(id));
/// ```
#[must_use]
pub fn shape_id_from_hex(hex: &str) -> Option<ShapeId> {
    let raw = u64::from_str_radix(hex, 16).ok()?;
    let key_data = KeyData::from_ffi(raw);
    let id: ShapeId = key_data.into();
    if id.is_null() {
        return None;
    }
    Some(id)
}

#[cfg(test)]
mod tests {
    //! Unit tests for metadata namespace policy and hex encoding helpers.

    use slotmap::{Key, KeyData};

    use super::{
        GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX, NamespacePolicyError,
        gauss_namespace_declaration, shape_id_from_hex, shape_id_to_hex, validate_namespace_policy,
    };
    use crate::model::ShapeId;
    use rstest::rstest;

    #[rstest]
    fn namespace_declaration_uses_canonical_prefix_and_uri() {
        let declaration = gauss_namespace_declaration();
        assert!(declaration.contains(GAUSS_METADATA_PREFIX));
        assert!(declaration.contains(GAUSS_METADATA_NAMESPACE));
    }

    #[rstest]
    fn validate_accepts_svg_without_gauss_metadata_usage() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg">
              <path d="M 0 0 L 1 1" />
            </svg>
        "#;

        let result = validate_namespace_policy(svg);
        assert_eq!(result, Ok(()));
    }

    #[rstest]
    fn validate_accepts_canonical_gauss_declaration_without_usage() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="https://gauss.dev/ns/metadata/1">
              <path d="M 0 0 L 1 1" />
            </svg>
        "#;

        let result = validate_namespace_policy(svg);
        assert_eq!(result, Ok(()));
    }

    #[rstest]
    fn validate_accepts_gauss_metadata_block_when_canonical_prefix_is_declared() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="https://gauss.dev/ns/metadata/1">
              <metadata>
                <gauss:editor version="1" />
              </metadata>
              <path d="M 0 0 L 1 1" />
            </svg>
        "#;

        let result = validate_namespace_policy(svg);
        assert_eq!(result, Ok(()));
    }

    #[rstest]
    fn validate_rejects_unexpected_gauss_namespace_binding() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="https://example.com/not-gauss">
              <path d="M 0 0 L 1 1" />
            </svg>
        "#;

        let result = validate_namespace_policy(svg);
        assert_eq!(
            result,
            Err(NamespacePolicyError::InvalidGaussNamespaceBinding(
                "https://example.com/not-gauss".to_owned()
            ))
        );
    }

    #[rstest]
    fn validate_rejects_gauss_namespace_usage_without_canonical_gauss_prefix_declaration() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:g="https://gauss.dev/ns/metadata/1">
              <metadata>
                <g:editor version="1" />
              </metadata>
              <path d="M 0 0 L 1 1" />
            </svg>
        "#;

        let result = validate_namespace_policy(svg);
        assert_eq!(
            result,
            Err(NamespacePolicyError::MissingGaussNamespaceDeclaration)
        );
    }

    #[rstest]
    fn validate_rejects_descendant_rebinding_of_gauss_prefix() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="https://gauss.dev/ns/metadata/1">
              <metadata xmlns:gauss="https://example.com/not-gauss">
                <gauss:editor version="1" />
              </metadata>
              <path d="M 0 0 L 1 1" />
            </svg>
        "#;

        let result = validate_namespace_policy(svg);
        assert_eq!(
            result,
            Err(NamespacePolicyError::InvalidGaussNamespaceBinding(
                "https://example.com/not-gauss".to_owned()
            ))
        );
    }

    // -- ShapeId hex encoding tests --

    #[rstest]
    fn shape_id_hex_round_trip() {
        let id: ShapeId = KeyData::from_ffi(0x0000_0001_0000_0000).into();
        let hex = shape_id_to_hex(id);
        assert_eq!(hex.len(), 16);
        let decoded = shape_id_from_hex(&hex);
        assert_eq!(decoded, Some(id));
    }

    #[rstest]
    fn shape_id_hex_is_zero_padded_lowercase() {
        let id: ShapeId = KeyData::from_ffi(0x0000_0001_0000_0000).into();
        let hex = shape_id_to_hex(id);
        assert_eq!(hex, "0000000100000000");
    }

    #[rstest]
    fn shape_id_from_hex_rejects_null_key() {
        let null_id = ShapeId::null();
        let null_hex = shape_id_to_hex(null_id);
        assert_eq!(shape_id_from_hex(&null_hex), None);
    }

    #[rstest]
    fn shape_id_from_hex_rejects_invalid_hex() {
        assert_eq!(shape_id_from_hex("zzzzzzzzzzzzzzzz"), None);
    }

    #[rstest]
    fn shape_id_from_hex_rejects_empty_string() {
        assert_eq!(shape_id_from_hex(""), None);
    }
}
