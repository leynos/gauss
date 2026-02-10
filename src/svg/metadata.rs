//! Gauss SVG metadata namespace policy.
//!
//! This module defines the canonical Gauss metadata namespace and validates
//! namespace usage in imported SVG documents.

use std::{error::Error, fmt};

/// Canonical namespace prefix for Gauss metadata.
pub const GAUSS_METADATA_PREFIX: &str = "gauss";
/// Canonical namespace URI for Gauss metadata.
pub const GAUSS_METADATA_NAMESPACE: &str = "https://gauss.dev/ns/metadata/1";

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
pub const fn gauss_namespace_declaration() -> &'static str {
    r#"xmlns:gauss="https://gauss.dev/ns/metadata/1""#
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

    let declared_uri = root.lookup_namespace_uri(Some(GAUSS_METADATA_PREFIX));
    if let Some(uri) = declared_uri
        && uri != GAUSS_METADATA_NAMESPACE
    {
        return Err(NamespacePolicyError::InvalidGaussNamespaceBinding(
            uri.to_owned(),
        ));
    }

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

#[cfg(test)]
mod tests {
    //! Unit tests for metadata namespace policy helpers.

    use super::{
        GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX, NamespacePolicyError,
        gauss_namespace_declaration, validate_namespace_policy,
    };
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
}
