//! Behaviour tests for metadata round-trip fidelity.

use gauss::model::{
    Anchor, Document, GaussAttribute, PaintStyle, PathGeom, ResourceStore, Rgba, SegmentKind,
    Shape, Vec2,
};
use gauss::svg::export::{CanvasSize, ExportOptions, export_svg_with_metadata};
use gauss::svg::import::{ImportedSvg, import_svg_with_resources};
use gauss::test_helpers::shape_id_from_seed;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

fn line_shape(seed: u128) -> Shape {
    Shape {
        id: shape_id_from_seed(seed),
        z: 0,
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(0.0, 0.0)),
                Anchor::new(Vec2::new(10.0, 10.0)),
            ],
            segments: vec![SegmentKind::Line],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

#[derive(Default)]
struct MetadataWorld {
    doc: Document,
    resources: ResourceStore,
    export_svg: Option<String>,
    import_result: Option<ImportedSvg>,
    metadata_block: Option<String>,
}

#[fixture]
fn world() -> MetadataWorld {
    MetadataWorld::default()
}

// ---------------------------------------------------------------------------
// Given
// ---------------------------------------------------------------------------

#[given("a document with a shape")]
fn given_document_with_shape(world: &mut MetadataWorld) {
    world.doc = Document::new();
    world.doc.append_shape(line_shape(42));
}

#[given("a document with a named shape")]
fn given_document_with_named_shape(world: &mut MetadataWorld) {
    world.doc = Document::new();
    let mut shape = line_shape(50);
    shape.name = Some("Test Shape".to_owned());
    world.doc.append_shape(shape);
}

#[given("a document with a locked and hidden shape")]
fn given_document_with_locked_hidden_shape(world: &mut MetadataWorld) {
    world.doc = Document::new();
    let mut shape = line_shape(60);
    shape.locked = true;
    shape.hidden = true;
    world.doc.append_shape(shape);
}

#[given("an SVG with unknown gauss attributes on a shape")]
fn given_svg_with_unknown_attrs(world: &mut MetadataWorld) {
    let mut doc = Document::new();
    let mut shape = line_shape(70);
    shape.gauss_metadata = vec![
        GaussAttribute::new("layer", "foreground"),
        GaussAttribute::new("opacity", "0.5"),
    ];
    doc.append_shape(shape);
    world.export_svg = Some(export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: None,
        web_ready: false,
    }));
}

#[given("an SVG with a metadata block")]
fn given_svg_with_metadata_block(world: &mut MetadataWorld) {
    let mut doc = Document::new();
    doc.append_shape(line_shape(80));
    let metadata = "\n<gauss:project>test-project</gauss:project>";
    world.metadata_block = Some(metadata.to_owned());
    world.export_svg = Some(export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: Some(metadata),
        web_ready: false,
    }));
}

#[given("a plain SVG without Gauss metadata")]
fn given_plain_svg(world: &mut MetadataWorld) {
    world.export_svg = Some(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
<path d="M 0 0 L 10 10" stroke="#000000" stroke-width="1" fill="none" />
</svg>
"##
        .to_owned(),
    );
}

// ---------------------------------------------------------------------------
// When
// ---------------------------------------------------------------------------

#[when("I export and re-import the document")]
fn when_export_and_reimport(world: &mut MetadataWorld) {
    let svg = export_svg_with_metadata(ExportOptions {
        doc: &world.doc,
        resources: &world.resources,
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: world.metadata_block.as_deref(),
        web_ready: false,
    });
    world.export_svg = Some(svg.clone());
    match import_svg_with_resources(&svg) {
        Ok(imported) => world.import_result = Some(imported),
        Err(err) => panic!("re-import should succeed: {err}"),
    }
}

#[when("I import and re-export the SVG")]
fn when_import_and_reexport(world: &mut MetadataWorld) -> TestSupportResult<()> {
    let svg = world
        .export_svg
        .clone()
        .ok_or_else(|| TestSupportError::missing("svg", "import step"))?;
    let imported = import_svg_with_resources(&svg)
        .map_err(|err| TestSupportError::expectation(format!("import failed: {err}")))?;
    let re_exported = export_svg_with_metadata(ExportOptions {
        doc: &imported.document,
        resources: &imported.resources,
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: imported.gauss_metadata_block.as_deref(),
        web_ready: false,
    });
    world.export_svg = Some(re_exported);
    world.import_result = Some(imported);
    Ok(())
}

#[when("I import the SVG")]
fn when_import_svg(world: &mut MetadataWorld) -> TestSupportResult<()> {
    let svg = world
        .export_svg
        .clone()
        .ok_or_else(|| TestSupportError::missing("svg", "import step"))?;
    let imported = import_svg_with_resources(&svg)
        .map_err(|err| TestSupportError::expectation(format!("import failed: {err}")))?;
    world.import_result = Some(imported);
    Ok(())
}

// ---------------------------------------------------------------------------
// Then
// ---------------------------------------------------------------------------

fn imported_shape(world: &MetadataWorld) -> TestSupportResult<&Shape> {
    let imported = world
        .import_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("import result", "assertion"))?;
    imported
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", "assertion"))
}

#[then("the imported shape retains its gauss:id")]
fn then_shape_retains_id(world: &MetadataWorld) -> TestSupportResult<()> {
    let shape = imported_shape(world)?;
    let expected = shape_id_from_seed(42);
    if shape.id != expected {
        return Err(TestSupportError::expectation(format!(
            "expected shape id {expected:?}, got {:?}",
            shape.id
        )));
    }
    Ok(())
}

#[then("the imported shape retains its name")]
fn then_shape_retains_name(world: &MetadataWorld) -> TestSupportResult<()> {
    let shape = imported_shape(world)?;
    if shape.name.as_deref() != Some("Test Shape") {
        return Err(TestSupportError::expectation(format!(
            "expected name 'Test Shape', got {:?}",
            shape.name
        )));
    }
    Ok(())
}

#[then("the imported shape retains its locked and hidden states")]
fn then_shape_retains_locked_hidden(world: &MetadataWorld) -> TestSupportResult<()> {
    let shape = imported_shape(world)?;
    if !shape.locked {
        return Err(TestSupportError::expectation("expected shape to be locked"));
    }
    if !shape.hidden {
        return Err(TestSupportError::expectation("expected shape to be hidden"));
    }
    Ok(())
}

#[then("the unknown gauss attributes are preserved")]
fn then_unknown_attrs_preserved(world: &MetadataWorld) -> TestSupportResult<()> {
    let shape = imported_shape(world)?;
    let expected = vec![
        GaussAttribute::new("layer", "foreground"),
        GaussAttribute::new("opacity", "0.5"),
    ];
    if shape.gauss_metadata != expected {
        return Err(TestSupportError::expectation(format!(
            "expected gauss_metadata {expected:?}, got {:?}",
            shape.gauss_metadata
        )));
    }
    Ok(())
}

#[then("the metadata block content is preserved")]
fn then_metadata_block_preserved(world: &MetadataWorld) -> TestSupportResult<()> {
    let imported = world
        .import_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("import result", "metadata block assertion"))?;
    let block = imported
        .gauss_metadata_block
        .as_deref()
        .ok_or_else(|| TestSupportError::expectation("expected metadata block to be present"))?;
    if !block.contains("<gauss:project>test-project</gauss:project>") {
        return Err(TestSupportError::expectation(format!(
            "expected metadata block to contain project element, got: {block}"
        )));
    }
    Ok(())
}

#[then("all shapes have default metadata")]
fn then_shapes_have_default_metadata(world: &MetadataWorld) -> TestSupportResult<()> {
    let imported = world
        .import_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("import result", "default metadata assertion"))?;
    let shape = imported
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", "default metadata assertion"))?;
    if shape.name.is_some() {
        return Err(TestSupportError::expectation(format!(
            "expected no name, got {:?}",
            shape.name
        )));
    }
    if shape.locked {
        return Err(TestSupportError::expectation(
            "expected shape not to be locked",
        ));
    }
    if shape.hidden {
        return Err(TestSupportError::expectation(
            "expected shape not to be hidden",
        ));
    }
    if !shape.gauss_metadata.is_empty() {
        return Err(TestSupportError::expectation(format!(
            "expected empty gauss_metadata, got {:?}",
            shape.gauss_metadata
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

#[scenario(
    path = "tests/features/metadata_round_trip.feature",
    name = "Shape IDs survive save and reload"
)]
fn shape_ids_survive_save_and_reload(world: MetadataWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/metadata_round_trip.feature",
    name = "Shape names survive round-trip"
)]
fn shape_names_survive_round_trip(world: MetadataWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/metadata_round_trip.feature",
    name = "Locked and hidden states survive round-trip"
)]
fn locked_and_hidden_states_survive_round_trip(world: MetadataWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/metadata_round_trip.feature",
    name = "Unknown Gauss attributes survive round-trip"
)]
fn unknown_gauss_attributes_survive_round_trip(world: MetadataWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/metadata_round_trip.feature",
    name = "Metadata block survives round-trip"
)]
fn metadata_block_survives_round_trip(world: MetadataWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/metadata_round_trip.feature",
    name = "Plain SVG without metadata imports cleanly"
)]
fn plain_svg_without_metadata_imports_cleanly(world: MetadataWorld) {
    let _ = world;
}
