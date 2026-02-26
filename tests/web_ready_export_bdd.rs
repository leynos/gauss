//! Behaviour tests for web-ready SVG export.

use gauss::model::{
    Document, GaussAttribute, Gradient, GradientId, GradientKind, GradientStop, LinearGradient,
    Paint, PaintStyle, PatternId, PatternResource, ResourceStore, Rgba, Shape, Vec2,
};
use gauss::svg::export::{
    CanvasSize, SvgExportError, export_svg_with_resources_web_ready,
    export_svg_with_resources_web_ready_checked,
};
use gauss::svg::import::{ImportedSvg, SvgImportError, import_svg_with_resources};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult, line_shape};

#[derive(Default)]
struct WebReadyWorld {
    doc: Document,
    resources: ResourceStore,
    export_svg: Option<String>,
    checked_export_result: Option<Result<String, SvgExportError>>,
    import_result: Option<Result<ImportedSvg, SvgImportError>>,
    expected_missing_gradient: Option<GradientId>,
    expected_missing_pattern: Option<PatternId>,
}

#[fixture]
fn world() -> WebReadyWorld {
    WebReadyWorld::default()
}

// ---------------------------------------------------------------------------
// Given
// ---------------------------------------------------------------------------

#[given("a document with Gauss metadata on a shape")]
fn given_document_with_gauss_metadata(world: &mut WebReadyWorld) {
    world.doc = Document::new();
    let mut shape = line_shape(420);
    shape.name = Some("Web-ready sample".to_owned());
    shape.locked = true;
    shape.hidden = true;
    shape.gauss_metadata = vec![GaussAttribute::new("role", "overlay")];
    world.doc.append_shape(shape);
    world.expected_missing_gradient = None;
    world.expected_missing_pattern = None;
}

#[given("a document with a dangling gradient paint reference")]
fn given_document_with_dangling_gradient(world: &mut WebReadyWorld) {
    world.doc = Document::new();
    world.resources = ResourceStore::new();

    let gradient_id = world.resources.insert_gradient(Gradient::new(
        "dangling-gradient",
        GradientKind::Linear(LinearGradient::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            vec![
                GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                GradientStop::new(1.0, Rgba::new(0, 0, 255, 255)),
            ],
        )),
    ));
    let _removed = world.resources.remove_gradient(gradient_id);

    let mut shape = line_shape(421);
    shape.style = PaintStyle::new_with_paint(Paint::gradient(gradient_id), 1.0, Paint::None);
    world.doc.append_shape(shape);
    world.expected_missing_gradient = Some(gradient_id);
    world.expected_missing_pattern = None;
}

#[given("a document with a dangling pattern paint reference")]
fn given_document_with_dangling_pattern(world: &mut WebReadyWorld) {
    world.doc = Document::new();
    world.resources = ResourceStore::new();

    let pattern_id = world.resources.insert_pattern(PatternResource::new(
        "dangling-pattern",
        r#"<circle cx="1" cy="1" r="1" />"#,
    ));
    let _removed = world.resources.remove_pattern(pattern_id);

    let mut shape = line_shape(422);
    shape.style = PaintStyle::new_with_paint(Paint::None, 1.0, Paint::pattern(pattern_id));
    world.doc.append_shape(shape);
    world.expected_missing_gradient = None;
    world.expected_missing_pattern = Some(pattern_id);
}

/// Helper to export web-ready SVG with optional validation.
fn export_web_ready_with_validation(world: &mut WebReadyWorld, checked: bool) {
    let canvas_size = CanvasSize::new(100.0, 100.0);

    if checked {
        let result =
            export_svg_with_resources_web_ready_checked(&world.doc, &world.resources, canvas_size);
        if let Ok(svg) = &result {
            world.export_svg = Some(svg.clone());
        }
        world.checked_export_result = Some(result);
    } else {
        let svg = export_svg_with_resources_web_ready(&world.doc, &world.resources, canvas_size);
        world.export_svg = Some(svg);
        world.checked_export_result = None;
    }
}

// ---------------------------------------------------------------------------
// When
// ---------------------------------------------------------------------------

#[when("I export the document as web-ready SVG")]
fn when_export_web_ready(world: &mut WebReadyWorld) {
    export_web_ready_with_validation(world, false);
}

#[when("I export the document as web-ready SVG with reference validation")]
fn when_export_web_ready_checked(world: &mut WebReadyWorld) {
    export_web_ready_with_validation(world, true);
}

#[when("I export and re-import the document as web-ready SVG")]
fn when_export_and_reimport_web_ready(world: &mut WebReadyWorld) {
    let svg = export_svg_with_resources_web_ready(
        &world.doc,
        &world.resources,
        CanvasSize::new(100.0, 100.0),
    );
    world.export_svg = Some(svg.clone());
    world.import_result = Some(import_svg_with_resources(&svg));
}

// ---------------------------------------------------------------------------
// Then
// ---------------------------------------------------------------------------

fn exported_svg(world: &WebReadyWorld) -> TestSupportResult<&str> {
    world
        .export_svg
        .as_deref()
        .ok_or_else(|| TestSupportError::missing("web-ready export", "assertion"))
}

fn imported_result(world: &WebReadyWorld) -> TestSupportResult<&ImportedSvg> {
    let result = world
        .import_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("import result", "assertion"))?;
    result
        .as_ref()
        .map_err(|error| TestSupportError::expectation(format!("import failed: {error}")))
}

fn imported_shape(world: &WebReadyWorld) -> TestSupportResult<&Shape> {
    imported_result(world)?
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", "assertion"))
}

#[then("the web-ready SVG strips Gauss metadata artefacts")]
fn then_web_ready_strips_gauss_metadata(world: &WebReadyWorld) -> TestSupportResult<()> {
    let svg = exported_svg(world)?;

    if !svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg""#) {
        return Err(TestSupportError::expectation(
            "expected SVG root declaration for web-ready output",
        ));
    }
    if !svg.contains(r#"<path d="M 0 0 L 10 10""#) {
        return Err(TestSupportError::expectation(
            "expected path geometry to remain in web-ready output",
        ));
    }
    if !svg.contains(r##" stroke="#000000" stroke-width="1" fill="none""##) {
        return Err(TestSupportError::expectation(
            "expected renderable stroke/fill style to remain in web-ready output",
        ));
    }
    if svg.contains("xmlns:gauss=") {
        return Err(TestSupportError::expectation(
            "expected web-ready output to omit gauss namespace declaration",
        ));
    }
    if svg.contains(" gauss:") {
        return Err(TestSupportError::expectation(
            "expected web-ready output to omit gauss:* path attributes",
        ));
    }
    if svg.contains("<metadata>") {
        return Err(TestSupportError::expectation(
            "expected web-ready output to omit metadata blocks",
        ));
    }

    Ok(())
}

#[then("the web-ready export fails with a missing gradient reference error")]
fn then_web_ready_reports_missing_gradient(world: &WebReadyWorld) -> TestSupportResult<()> {
    let expected_gradient = world.expected_missing_gradient.ok_or_else(|| {
        TestSupportError::missing("expected missing gradient id", "error assertion")
    })?;
    let result = world
        .checked_export_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("checked export result", "error assertion"))?;

    match result {
        Err(SvgExportError::MissingGradientReference(actual)) if *actual == expected_gradient => {
            Ok(())
        }
        Err(other) => Err(TestSupportError::expectation(format!(
            "expected MissingGradientReference({expected_gradient:?}), got {other:?}"
        ))),
        Ok(_) => Err(TestSupportError::expectation(
            "expected checked web-ready export to fail for missing gradient reference",
        )),
    }
}

#[then("the web-ready export fails with a missing pattern reference error")]
fn then_web_ready_reports_missing_pattern(world: &WebReadyWorld) -> TestSupportResult<()> {
    let expected_pattern = world.expected_missing_pattern.ok_or_else(|| {
        TestSupportError::missing("expected missing pattern id", "error assertion")
    })?;
    let result = world
        .checked_export_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("checked export result", "error assertion"))?;

    match result {
        Err(SvgExportError::MissingPatternReference(actual)) if *actual == expected_pattern => {
            Ok(())
        }
        Err(other) => Err(TestSupportError::expectation(format!(
            "expected MissingPatternReference({expected_pattern:?}), got {other:?}"
        ))),
        Ok(_) => Err(TestSupportError::expectation(
            "expected checked web-ready export to fail for missing pattern reference",
        )),
    }
}

#[then("the imported web-ready shape has default metadata values")]
fn then_imported_shape_has_default_metadata(world: &WebReadyWorld) -> TestSupportResult<()> {
    let imported = imported_result(world)?;
    if imported.gauss_metadata_block.is_some() {
        return Err(TestSupportError::expectation(
            "expected web-ready import to have no metadata block",
        ));
    }

    let shape = imported_shape(world)?;
    if shape.name.is_some() {
        return Err(TestSupportError::expectation(format!(
            "expected no imported shape name, got {:?}",
            shape.name
        )));
    }
    if shape.locked {
        return Err(TestSupportError::expectation(
            "expected imported shape to be unlocked",
        ));
    }
    if shape.hidden {
        return Err(TestSupportError::expectation(
            "expected imported shape to be visible",
        ));
    }
    if !shape.gauss_metadata.is_empty() {
        return Err(TestSupportError::expectation(format!(
            "expected no imported gauss metadata attrs, got {:?}",
            shape.gauss_metadata
        )));
    }
    Ok(())
}

#[then("the imported web-ready shape keeps renderable geometry and paint style")]
fn then_imported_shape_keeps_renderable_data(world: &WebReadyWorld) -> TestSupportResult<()> {
    let shape = imported_shape(world)?;
    let expected = line_shape(420);

    if shape.path != expected.path {
        return Err(TestSupportError::expectation(format!(
            "expected imported path {:?}, got {:?}",
            expected.path, shape.path
        )));
    }
    if shape.style != expected.style {
        return Err(TestSupportError::expectation(format!(
            "expected imported style {:?}, got {:?}",
            expected.style, shape.style
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

#[scenario(
    path = "tests/features/web_ready_export.feature",
    name = "Web-ready export strips Gauss metadata artefacts"
)]
fn web_ready_export_strips_gauss_metadata_artefacts(world: WebReadyWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/web_ready_export.feature",
    name = "Checked web-ready export reports missing gradient references"
)]
fn checked_web_ready_export_reports_missing_gradient_references(world: WebReadyWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/web_ready_export.feature",
    name = "Checked web-ready export reports missing pattern references"
)]
fn checked_web_ready_export_reports_missing_pattern_references(world: WebReadyWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/web_ready_export.feature",
    name = "Web-ready output imports as plain SVG metadata defaults"
)]
fn web_ready_output_imports_as_plain_svg_metadata_defaults(world: WebReadyWorld) {
    let _ = world;
}
