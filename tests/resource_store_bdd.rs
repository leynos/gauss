//! Behaviour tests for resource stores and paint references.

use gauss::model::{
    Anchor, Document, Gradient, GradientKind, GradientStop, LinearGradient, Paint, PaintStyle,
    PathGeom, PatternResource, ResourceStore, SegmentKind, Shape, Vec2,
};
use gauss::svg::export::export_svg_with_resources;
use gauss::svg::import::{ImportedSvg, SvgImportError, import_svg_with_resources};
use gauss::svg::metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::shapes::shape_id;
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct ResourceWorld {
    doc: Document,
    resources: ResourceStore,
    export_svg: Option<String>,
    import_result: Option<Result<ImportedSvg, SvgImportError>>,
}

#[fixture]
fn world() -> ResourceWorld {
    ResourceWorld::default()
}

#[given("a document with a gradient stroke and pattern fill")]
fn given_document_with_resource_paints(world: &mut ResourceWorld) {
    let gradient_id = world.resources.insert_gradient(Gradient::new(
        "sunset",
        GradientKind::Linear(LinearGradient::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            vec![
                GradientStop::new(0.0, gauss::model::Rgba::new(255, 0, 0, 255)),
                GradientStop::new(1.0, gauss::model::Rgba::new(255, 255, 0, 255)),
            ],
        )),
    ));
    let pattern_id = world.resources.insert_pattern(PatternResource::new(
        "dots",
        "<circle cx=\"1\" cy=\"1\" r=\"1\" />",
    ));

    let shape = Shape {
        id: shape_id(200),
        z: 0,
        style: PaintStyle::new_with_paint(
            Paint::gradient(gradient_id),
            2.0,
            Paint::pattern(pattern_id),
        ),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(0.0, 0.0)),
                Anchor::new(Vec2::new(5.0, 0.0)),
                Anchor::new(Vec2::new(5.0, 5.0)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
    };

    world.doc = Document::new();
    world.doc.append_shape(shape);
}

#[given("an SVG path that references an unknown gradient")]
fn given_svg_with_unknown_resource(world: &mut ResourceWorld) {
    world.export_svg = Some(
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="url(#missing)" stroke-width="1" fill="none" />
        </svg>
        "#
        .to_owned(),
    );
}

#[given("an SVG metadata block using Gauss URI without the gauss prefix declaration")]
fn given_svg_with_gauss_uri_without_gauss_prefix(world: &mut ResourceWorld) {
    world.export_svg = Some(format!(
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:g="{GAUSS_METADATA_NAMESPACE}">
          <metadata><g:editor version="1" /></metadata>
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
        "##
    ));
}

#[when("I export and re-import the document with resources")]
fn when_export_and_reimport(world: &mut ResourceWorld) {
    let svg = export_svg_with_resources(&world.doc, &world.resources, 100.0, 100.0);
    world.export_svg = Some(svg.clone());
    world.import_result = Some(import_svg_with_resources(&svg));
}

#[when("I import the SVG with resources")]
fn when_import_svg(world: &mut ResourceWorld) -> TestSupportResult<()> {
    let svg = world
        .export_svg
        .clone()
        .ok_or_else(|| TestSupportError::missing("svg", "import step"))?;
    world.import_result = Some(import_svg_with_resources(&svg));
    Ok(())
}

#[then("the imported document should keep gradient and pattern paint references")]
fn then_resource_refs_round_trip(world: &ResourceWorld) -> TestSupportResult<()> {
    let imported = world
        .import_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("import result", "round-trip assertion"))?
        .as_ref()
        .map_err(|error| TestSupportError::expectation(format!("import failed: {error}")))?;

    if imported.resources.gradient_count() != 1 || imported.resources.pattern_count() != 1 {
        return Err(TestSupportError::expectation(format!(
            "expected one gradient and one pattern, got {} gradients and {} patterns",
            imported.resources.gradient_count(),
            imported.resources.pattern_count()
        )));
    }

    let shape = imported
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", "round-trip assertion"))?;

    let Some(gradient_id) = imported.resources.gradient_id_for_svg_id("sunset") else {
        return Err(TestSupportError::expectation(
            "expected sunset gradient id in imported resources",
        ));
    };

    let Some(pattern_id) = imported.resources.pattern_id_for_svg_id("dots") else {
        return Err(TestSupportError::expectation(
            "expected dots pattern id in imported resources",
        ));
    };

    if shape.style.stroke != Paint::gradient(gradient_id) {
        return Err(TestSupportError::expectation(format!(
            "expected gradient stroke reference, got {:?}",
            shape.style.stroke
        )));
    }

    if shape.style.fill != Paint::pattern(pattern_id) {
        return Err(TestSupportError::expectation(format!(
            "expected pattern fill reference, got {:?}",
            shape.style.fill
        )));
    }

    Ok(())
}

/// Helper function to assert that an import fails with a specific error.
#[expect(
    clippy::too_many_arguments,
    reason = "BDD assertions pass explicit context and matcher for clear failures"
)]
fn assert_import_error<F>(
    world: &ResourceWorld,
    assertion_context: &str,
    expected_error_desc: &str,
    success_failure_msg: &str,
    error_matcher: F,
) -> TestSupportResult<()>
where
    F: FnOnce(&SvgImportError) -> bool,
{
    let result = world
        .import_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("import result", assertion_context))?;

    match result {
        Err(error) if error_matcher(error) => Ok(()),
        Err(other) => Err(TestSupportError::expectation(format!(
            "expected {expected_error_desc}, got {other}"
        ))),
        Ok(_) => Err(TestSupportError::expectation(success_failure_msg)),
    }
}

#[then("the import should fail with a missing referenced resource error")]
fn then_missing_resource_error(world: &ResourceWorld) -> TestSupportResult<()> {
    assert_import_error(
        world,
        "error assertion",
        "MissingReferencedResource('missing')",
        "expected import to fail for missing referenced resource",
        |error| matches!(error, SvgImportError::MissingReferencedResource(id) if id == "missing"),
    )
}

#[then("the exported SVG should declare the canonical Gauss metadata namespace")]
fn then_exported_svg_declares_namespace(world: &ResourceWorld) -> TestSupportResult<()> {
    let svg = world
        .export_svg
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("exported svg", "namespace assertion"))?;
    let expected = format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#);
    if !svg.contains(expected.as_str()) {
        return Err(TestSupportError::expectation(format!(
            "expected exported SVG to include '{expected}', got: {svg}"
        )));
    }
    Ok(())
}

#[then("the import should fail with a missing gauss namespace declaration error")]
fn then_missing_gauss_namespace_error(world: &ResourceWorld) -> TestSupportResult<()> {
    assert_import_error(
        world,
        "namespace error assertion",
        "MissingGaussNamespaceDeclaration",
        "expected import to fail for missing gauss namespace declaration",
        |error| matches!(error, SvgImportError::MissingGaussNamespaceDeclaration),
    )
}

#[scenario(
    path = "tests/features/resource_store.feature",
    name = "Gradient and pattern references round-trip through SVG"
)]
fn gradient_and_pattern_references_round_trip_through_svg(world: ResourceWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/resource_store.feature",
    name = "Missing resource reference is rejected"
)]
fn missing_resource_reference_is_rejected(world: ResourceWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/resource_store.feature",
    name = "Exported SVG declares Gauss metadata namespace"
)]
fn exported_svg_declares_gauss_metadata_namespace(world: ResourceWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/resource_store.feature",
    name = "Gauss metadata usage requires canonical gauss prefix declaration"
)]
fn gauss_metadata_usage_requires_canonical_prefix(world: ResourceWorld) {
    let _ = world;
}
