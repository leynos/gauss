//! Namespace policy tests for SVG import/export.

use super::*;

#[rstest]
fn imports_svg_with_canonical_gauss_namespace_metadata_block() {
    let svg = format!(
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}">
          <metadata>
            <{GAUSS_METADATA_PREFIX}:editor version="1" />
          </metadata>
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##
    );

    if let Err(error) = import_svg_with_resources(svg.as_str()) {
        panic!("canonical gauss metadata should import: {error}");
    }
}

#[rstest]
#[case::with_resources(true)]
#[case::without_resources(false)]
fn rejects_invalid_gauss_prefix_binding(#[case] with_resources: bool) {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="https://example.com/not-gauss">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;

    let result = if with_resources {
        import_svg_with_resources(svg).map(|imported| imported.document)
    } else {
        import_svg(svg)
    };

    assert_eq!(
        result,
        Err(SvgImportError::InvalidGaussNamespaceBinding(
            "https://example.com/not-gauss".to_owned()
        ))
    );
}

#[rstest]
fn reports_gauss_namespace_usage_without_canonical_prefix_declaration() {
    let svg = format!(
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:g="{GAUSS_METADATA_NAMESPACE}">
          <metadata>
            <g:editor version="1" />
          </metadata>
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##
    );

    let result = import_svg_with_resources(svg.as_str());
    assert_eq!(
        result,
        Err(SvgImportError::MissingGaussNamespaceDeclaration)
    );
}

#[rstest]
fn exported_svg_declares_canonical_gauss_namespace() {
    let exported = export_svg_with_resources(
        &Document::new(),
        &ResourceStore::new(),
        CanvasSize::new(10.0, 10.0),
    );
    let namespace = format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#);
    assert!(exported.contains(namespace.as_str()));
}
