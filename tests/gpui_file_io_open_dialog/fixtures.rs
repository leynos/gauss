//! Given steps that create the temporary SVG variants used by Open scenarios.

use gauss::svg::metadata::GAUSS_METADATA_NAMESPACE;
use gpui::TestAppContext;
use rstest_bdd_macros::given;
use test_support::TestSupportError;

use super::prepare_svg;

#[given("a temporary plain SVG")]
fn plain_svg(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_svg(
        cx,
        "gauss-test-open",
        r##"<svg xmlns="http://www.w3.org/2000/svg">
<path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
</svg>"##,
    )
}

#[given("a temporary SVG with gradient and pattern resources")]
fn svg_with_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_svg(
        cx,
        "gauss-test-open-resources",
        r##"<svg xmlns="http://www.w3.org/2000/svg"><defs>
<linearGradient id="sunset" x1="0" y1="0" x2="1" y2="0">
<stop offset="0" stop-color="#ff0000"/><stop offset="1" stop-color="#ffff00"/>
</linearGradient><pattern id="dots"><circle cx="1" cy="1" r="1"/></pattern>
</defs><path d="M 1 2 L 3 4 Z" stroke="url(#sunset)" fill="url(#dots)"/></svg>"##,
    )
}

#[given("a temporary SVG with a missing resource reference")]
fn svg_with_missing_resource(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_svg(
        cx,
        "gauss-test-open-invalid",
        r##"<svg xmlns="http://www.w3.org/2000/svg">
<path d="M 1 2 L 3 4" stroke="#000000" fill="url(#missing)" />
</svg>"##,
    )
}

#[given("a temporary SVG with the canonical Gauss metadata namespace")]
fn svg_with_canonical_namespace(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_svg(
        cx,
        "gauss-test-open-gauss-metadata",
        &format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="{GAUSS_METADATA_NAMESPACE}">
<metadata><gauss:editor version="1" /></metadata>
<path d="M 1 2 L 3 4" stroke="#000000" fill="none" /></svg>"##
        ),
    )
}

#[given("a temporary SVG with a non-canonical Gauss metadata prefix")]
fn svg_with_noncanonical_prefix(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_svg(
        cx,
        "gauss-test-open-invalid-gauss",
        &format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:g="{GAUSS_METADATA_NAMESPACE}">
<metadata><g:editor version="1" /></metadata>
<path d="M 1 2 L 3 4" stroke="#000000" fill="none" /></svg>"##
        ),
    )
}
