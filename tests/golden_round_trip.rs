//! Golden reference tests for SVG metadata round-trip fidelity.
//!
//! Each test constructs a document, exports it, and verifies:
//! 1. The exported SVG matches the golden reference file.
//! 2. Re-importing and re-exporting produces identical output (idempotency).

use camino::Utf8Path;
use cap_std::{ambient_authority, fs_utf8::Dir};
use gauss::model::{
    Anchor, Document, GaussAttribute, PaintStyle, PathGeom, ResourceStore, Rgba, SegmentKind, Shape,
};
use gauss::svg::export::{
    CanvasSize, ExportOptions, export_svg_with_metadata, export_svg_with_resources_web_ready,
};
use gauss::svg::import::import_svg_with_resources;
use gauss::test_helpers::shape_id_from_seed;
use rstest::{fixture, rstest};
use test_support::{TestSupportError, TestSupportResult};

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[fixture]
fn triangle_path() -> PathGeom {
    PathGeom {
        anchors: vec![
            Anchor::new(gauss::model::Vec2::new(10.0, 10.0)),
            Anchor::new(gauss::model::Vec2::new(90.0, 10.0)),
            Anchor::new(gauss::model::Vec2::new(50.0, 80.0)),
        ],
        segments: vec![SegmentKind::Line, SegmentKind::Line],
        closed: true,
        closing_segment: SegmentKind::Line,
    }
}

#[fixture]
fn default_style() -> PaintStyle {
    PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None)
}

#[fixture]
fn minimal_test_shape(
    #[default(1)] seed: u128,
    triangle_path: PathGeom,
    default_style: PaintStyle,
) -> Shape {
    Shape {
        id: shape_id_from_seed(seed),
        z: 0,
        style: default_style,
        path: triangle_path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

fn golden_dir() -> TestSupportResult<Dir> {
    Dir::open_ambient_dir(GOLDEN_DIR, ambient_authority())
        .map_err(|err| TestSupportError::io("open golden dir", err))
}

fn read_golden(name: &str) -> TestSupportResult<String> {
    let dir = golden_dir()?;
    let file_name = format!("{name}.svg");
    dir.read_to_string(Utf8Path::new(&file_name))
        .map_err(|err| TestSupportError::io("read golden file", err))
}

fn assert_golden(name: &str, svg: &str) -> TestSupportResult<()> {
    let expected = read_golden(name)?;
    if svg != expected {
        return Err(TestSupportError::expectation(format!(
            "SVG output does not match golden file {name}"
        )));
    }
    Ok(())
}

fn assert_idempotent(svg: &str, metadata_block: Option<&str>) -> TestSupportResult<()> {
    let imported = import_svg_with_resources(svg)
        .map_err(|err| TestSupportError::expectation(format!("re-import failed: {err}")))?;
    let re_exported = export_svg_with_metadata(ExportOptions {
        doc: &imported.document,
        resources: &imported.resources,
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: imported.gauss_metadata_block.as_deref().or(metadata_block),
    });
    if svg != re_exported {
        return Err(TestSupportError::expectation(
            "round-trip should be idempotent",
        ));
    }
    Ok(())
}

#[fixture]
fn export_doc() -> impl Fn(&Document, Option<&str>) -> String {
    |doc: &Document, metadata_block: Option<&str>| {
        export_svg_with_metadata(ExportOptions {
            doc,
            resources: &ResourceStore::new(),
            canvas_size: CanvasSize::new(100.0, 100.0),
            metadata_block,
        })
    }
}

fn assert_shape_round_trip(
    shape: Shape,
    golden_name: &str,
    export_doc: &impl Fn(&Document, Option<&str>) -> String,
) -> TestSupportResult<()> {
    let mut doc = Document::new();
    doc.append_shape(shape);
    let svg = export_doc(&doc, None);
    assert_golden(golden_name, &svg)?;
    assert_idempotent(&svg, None)
}

#[rstest]
fn plain_svg_without_gauss_metadata(
    triangle_path: PathGeom,
    default_style: PaintStyle,
    export_doc: impl Fn(&Document, Option<&str>) -> String,
) -> TestSupportResult<()> {
    let mut doc = Document::new();
    let shape = Shape {
        id: gauss::model::ShapeId::default(),
        z: 0,
        style: default_style,
        path: triangle_path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    };
    doc.append_shape(shape);

    let svg = export_doc(&doc, None);

    // Plain SVGs still get gauss:id from append_shape allocation, so
    // this test checks overall format rather than an exact golden match.
    let imported = import_svg_with_resources(&svg)
        .map_err(|err| TestSupportError::expectation(format!("re-import failed: {err}")))?;
    if imported.document.len() != 1 {
        return Err(TestSupportError::expectation(
            "imported document should have 1 shape",
        ));
    }
    assert_idempotent(&svg, None)
}

#[rstest]
#[case::with_id(minimal_test_shape(1, triangle_path(), default_style()), "with_id")]
#[case::with_full_metadata(
    Shape {
        name: Some("My Triangle".to_owned()),
        locked: true,
        hidden: true,
        ..minimal_test_shape(2, triangle_path(), default_style())
    },
    "with_full_metadata",
)]
#[case::with_unknown_attrs(
    Shape {
        gauss_metadata: vec![
            GaussAttribute::new("layer", "foreground"),
            GaussAttribute::new("opacity", "0.5"),
        ],
        ..minimal_test_shape(4, triangle_path(), default_style())
    },
    "with_unknown_attrs",
)]
fn shape_round_trip_variants(
    #[case] shape: Shape,
    #[case] golden_name: &str,
    export_doc: impl Fn(&Document, Option<&str>) -> String,
) -> TestSupportResult<()> {
    assert_shape_round_trip(shape, golden_name, &export_doc)
}

#[rstest]
fn shape_with_metadata_block(
    triangle_path: PathGeom,
    default_style: PaintStyle,
    export_doc: impl Fn(&Document, Option<&str>) -> String,
) -> TestSupportResult<()> {
    let id = shape_id_from_seed(3);
    let mut doc = Document::new();
    doc.append_shape(Shape {
        id,
        z: 0,
        style: default_style,
        path: triangle_path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    });

    let metadata = concat!(
        "\n<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">",
        "\n  <rdf:Description rdf:about=\"\"/>",
        "\n</rdf:RDF>"
    );
    let svg = export_doc(&doc, Some(metadata));
    assert_golden("with_metadata_block", &svg)?;
    assert_idempotent(&svg, None)
}

#[rstest]
fn full_round_trip_combined(
    triangle_path: PathGeom,
    default_style: PaintStyle,
    export_doc: impl Fn(&Document, Option<&str>) -> String,
) -> TestSupportResult<()> {
    let mut doc = Document::new();
    let id1 = shape_id_from_seed(10);
    let id2 = shape_id_from_seed(11);

    doc.append_shape(Shape {
        id: id1,
        z: 0,
        style: PaintStyle::new(
            Some(Rgba::new(255, 0, 0, 255)),
            2.0,
            Some(Rgba::new(0, 0, 255, 128)),
        ),
        path: triangle_path,
        name: Some("Red Triangle".to_owned()),
        locked: false,
        hidden: false,
        gauss_metadata: vec![GaussAttribute::new("custom", "value1")],
    });

    doc.append_shape(Shape {
        id: id2,
        z: 1,
        style: default_style,
        path: PathGeom {
            anchors: vec![
                Anchor::new(gauss::model::Vec2::new(20.0, 20.0)),
                Anchor::new(gauss::model::Vec2::new(80.0, 80.0)),
            ],
            segments: vec![SegmentKind::Line],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: true,
        hidden: false,
        gauss_metadata: Vec::new(),
    });

    let metadata = "\n<gauss:project>test-project</gauss:project>";
    let svg = export_doc(&doc, Some(metadata));
    assert_golden("full_round_trip", &svg)?;
    assert_idempotent(&svg, None)
}

fn assert_no_gauss_markers(svg: &str) -> TestSupportResult<()> {
    if svg.contains("gauss:") {
        return Err(TestSupportError::expectation(
            "web-ready export should omit all gauss metadata",
        ));
    }
    if svg.contains("xmlns:gauss=") {
        return Err(TestSupportError::expectation(
            "web-ready export should omit all gauss metadata",
        ));
    }
    if svg.contains("<metadata>") {
        return Err(TestSupportError::expectation(
            "web-ready export should omit all gauss metadata",
        ));
    }
    Ok(())
}

fn assert_default_shape_metadata(
    shape: &Shape,
    gauss_metadata_block: Option<&String>,
) -> TestSupportResult<()> {
    if shape.name.is_some() {
        return Err(TestSupportError::expectation(
            "web-ready export should import with default shape metadata flags",
        ));
    }
    if shape.locked {
        return Err(TestSupportError::expectation(
            "web-ready export should import with default shape metadata flags",
        ));
    }
    if shape.hidden {
        return Err(TestSupportError::expectation(
            "web-ready export should import with default shape metadata flags",
        ));
    }
    if !shape.gauss_metadata.is_empty() || gauss_metadata_block.is_some() {
        return Err(TestSupportError::expectation(
            "web-ready export should not contain any gauss metadata payload",
        ));
    }
    Ok(())
}

#[rstest]
fn web_ready_export_strips_gauss_metadata(
    triangle_path: PathGeom,
    default_style: PaintStyle,
) -> TestSupportResult<()> {
    let mut doc = Document::new();
    doc.append_shape(Shape {
        id: shape_id_from_seed(40),
        z: 0,
        style: default_style,
        path: triangle_path,
        name: Some("Triangle".to_owned()),
        locked: true,
        hidden: true,
        gauss_metadata: vec![GaussAttribute::new("layer", "foreground")],
    });
    let svg = export_svg_with_resources_web_ready(
        &doc,
        &ResourceStore::new(),
        CanvasSize::new(100.0, 100.0),
    );
    assert_golden("web_ready_strips_metadata", &svg)?;
    assert_no_gauss_markers(&svg)?;
    let imported = import_svg_with_resources(&svg)
        .map_err(|err| TestSupportError::expectation(format!("re-import failed: {err}")))?;
    let imported_shape = imported
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", "web-ready assertion"))?;
    assert_default_shape_metadata(imported_shape, imported.gauss_metadata_block.as_ref())?;
    Ok(())
}
