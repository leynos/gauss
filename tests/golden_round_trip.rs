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
use gauss::svg::export::{ExportOptions, export_svg_with_metadata};
use gauss::svg::import::import_svg_with_resources;
use gauss::test_helpers::shape_id_from_seed;
use rstest::rstest;
use test_support::{TestSupportError, TestSupportResult};

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

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

const fn default_style() -> PaintStyle {
    PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None)
}

fn minimal_test_shape(seed: u128) -> Shape {
    Shape {
        id: shape_id_from_seed(seed),
        z: 0,
        style: default_style(),
        path: triangle_path(),
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
        canvas_width: 100.0,
        canvas_height: 100.0,
        metadata_block: imported.gauss_metadata_block.as_deref().or(metadata_block),
    });
    if svg != re_exported {
        return Err(TestSupportError::expectation(
            "round-trip should be idempotent",
        ));
    }
    Ok(())
}

fn export_doc(doc: &Document, metadata_block: Option<&str>) -> String {
    export_svg_with_metadata(ExportOptions {
        doc,
        resources: &ResourceStore::new(),
        canvas_width: 100.0,
        canvas_height: 100.0,
        metadata_block,
    })
}

fn assert_shape_round_trip(shape: Shape, golden_name: &str) -> TestSupportResult<()> {
    let mut doc = Document::new();
    doc.append_shape(shape);
    let svg = export_doc(&doc, None);
    assert_golden(golden_name, &svg)?;
    assert_idempotent(&svg, None)
}

#[rstest]
fn plain_svg_without_gauss_metadata() -> TestSupportResult<()> {
    let mut doc = Document::new();
    let shape = Shape {
        id: gauss::model::ShapeId::default(),
        z: 0,
        style: default_style(),
        path: triangle_path(),
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
fn shape_with_gauss_id() -> TestSupportResult<()> {
    assert_shape_round_trip(minimal_test_shape(1), "with_id")
}

#[rstest]
fn shape_with_full_metadata() -> TestSupportResult<()> {
    assert_shape_round_trip(
        Shape {
            name: Some("My Triangle".to_owned()),
            locked: true,
            hidden: true,
            ..minimal_test_shape(2)
        },
        "with_full_metadata",
    )
}

#[rstest]
fn shape_with_metadata_block() -> TestSupportResult<()> {
    let id = shape_id_from_seed(3);
    let mut doc = Document::new();
    doc.append_shape(Shape {
        id,
        z: 0,
        style: default_style(),
        path: triangle_path(),
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    });

    let metadata = "\n<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\
                     \n  <rdf:Description rdf:about=\"\"/>\
                     \n</rdf:RDF>";
    let svg = export_doc(&doc, Some(metadata));
    assert_golden("with_metadata_block", &svg)?;
    assert_idempotent(&svg, None)
}

#[rstest]
fn shape_with_unknown_gauss_attrs() -> TestSupportResult<()> {
    assert_shape_round_trip(
        Shape {
            gauss_metadata: vec![
                GaussAttribute::new("layer", "foreground"),
                GaussAttribute::new("opacity", "0.5"),
            ],
            ..minimal_test_shape(4)
        },
        "with_unknown_attrs",
    )
}

#[rstest]
fn full_round_trip_combined() -> TestSupportResult<()> {
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
        path: triangle_path(),
        name: Some("Red Triangle".to_owned()),
        locked: false,
        hidden: false,
        gauss_metadata: vec![GaussAttribute::new("custom", "value1")],
    });

    doc.append_shape(Shape {
        id: id2,
        z: 1,
        style: default_style(),
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
