//! Tests for Gauss metadata import.

use slotmap::Key;

use super::*;
use crate::model::GaussAttribute;
use crate::svg::export::{CanvasSize, ExportOptions, export_svg_with_metadata};
use crate::svg::metadata::shape_id_to_hex;
use crate::test_helpers::shape_id_from_seed;
use rstest::{fixture, rstest};

const GAUSS_NS_DECL: &str = r#"xmlns:gauss="https://gauss.dev/ns/metadata/1""#;

fn svg_with_path(path_attrs: &str) -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" {GAUSS_NS_DECL} width="100" height="100" viewBox="0 0 100 100">
<path d="M 0 0 L 10 10" stroke="#000000" stroke-width="1" fill="none"{path_attrs} />
</svg>
"##
    )
}

fn import_ok(svg: &str) -> crate::svg::import::ImportedSvg {
    match import_svg_with_resources(svg) {
        Ok(imported) => imported,
        Err(err) => panic!("import should succeed: {err}"),
    }
}

fn first_shape(imported: &crate::svg::import::ImportedSvg) -> &crate::model::Shape {
    let Some(shape) = imported.document.shape_at(0) else {
        panic!("should have imported a shape");
    };
    shape
}

fn export_and_reimport(
    doc: &Document,
    metadata_block: Option<&str>,
) -> crate::svg::import::ImportedSvg {
    let svg = export_svg_with_metadata(ExportOptions {
        doc,
        resources: &ResourceStore::new(),
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block,
        web_ready: false,
    });
    match import_svg_with_resources(&svg) {
        Ok(imported) => imported,
        Err(err) => panic!("re-import should succeed: {err}"),
    }
}

#[fixture]
fn line_shape(#[default(1)] seed: u128) -> Shape {
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

#[rstest]
fn imports_gauss_id_from_path_element() {
    let id = shape_id_from_seed(42);
    let hex = shape_id_to_hex(id);
    let svg = svg_with_path(&format!(r#" gauss:id="{hex}""#));
    let imported = import_ok(&svg);
    assert_eq!(first_shape(&imported).id, id);
}

#[rstest]
fn imports_shape_name() {
    let svg = svg_with_path(r#" gauss:id="ffffffff00000001" gauss:name="Layer 1""#);
    let imported = import_ok(&svg);
    assert_eq!(first_shape(&imported).name.as_deref(), Some("Layer 1"));
}

#[rstest]
fn imports_locked_state() {
    let svg = svg_with_path(r#" gauss:id="ffffffff00000001" gauss:locked="true""#);
    let imported = import_ok(&svg);
    assert!(first_shape(&imported).locked);
}

#[rstest]
fn imports_hidden_state() {
    let svg = svg_with_path(r#" gauss:id="ffffffff00000001" gauss:hidden="true""#);
    let imported = import_ok(&svg);
    assert!(first_shape(&imported).hidden);
}

#[rstest]
fn imports_unknown_gauss_attributes() {
    let svg = svg_with_path(
        r#" gauss:id="ffffffff00000001" gauss:layer="foreground" gauss:opacity="0.5""#,
    );
    let imported = import_ok(&svg);
    assert_eq!(
        first_shape(&imported).gauss_metadata,
        vec![
            GaussAttribute::new("layer", "foreground"),
            GaussAttribute::new("opacity", "0.5"),
        ]
    );
}

#[rstest]
fn imports_metadata_block() {
    let svg = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" {GAUSS_NS_DECL} width="100" height="100" viewBox="0 0 100 100">
<metadata>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""/>
</rdf:RDF>
</metadata>
<path d="M 0 0 L 10 10" stroke="#000000" stroke-width="1" fill="none" gauss:id="ffffffff00000001" />
</svg>
"##
    );
    let imported = import_ok(&svg);
    assert!(imported.gauss_metadata_block.is_some());
    let block = imported.gauss_metadata_block.as_deref().unwrap_or("");
    assert!(block.contains("rdf:RDF"));
}

#[rstest]
fn shape_without_gauss_id_gets_default() {
    let svg = svg_with_path("");
    let imported = import_ok(&svg);
    // Shape gets a fresh allocation via ensure_shape_id, not null.
    assert!(!first_shape(&imported).id.is_null());
}

#[rstest]
fn round_trip_preserves_gauss_id(#[with(99)] line_shape: Shape) {
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    let imported = export_and_reimport(&doc, None);
    assert_eq!(first_shape(&imported).id, shape_id_from_seed(99));
}

#[rstest]
fn round_trip_preserves_shape_name(#[with(100)] mut line_shape: Shape) {
    line_shape.name = Some("Test Name".to_owned());
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    let imported = export_and_reimport(&doc, None);
    assert_eq!(first_shape(&imported).name.as_deref(), Some("Test Name"));
}

#[rstest]
fn round_trip_preserves_locked_hidden(#[with(101)] mut line_shape: Shape) {
    line_shape.locked = true;
    line_shape.hidden = true;
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    let imported = export_and_reimport(&doc, None);
    let s = first_shape(&imported);
    assert!(s.locked);
    assert!(s.hidden);
}

#[rstest]
fn round_trip_preserves_unknown_attrs(#[with(102)] mut line_shape: Shape) {
    line_shape.gauss_metadata = vec![
        GaussAttribute::new("layer", "bg"),
        GaussAttribute::new("custom-key", "custom-val"),
    ];
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    let imported = export_and_reimport(&doc, None);
    assert_eq!(
        first_shape(&imported).gauss_metadata,
        vec![
            GaussAttribute::new("layer", "bg"),
            GaussAttribute::new("custom-key", "custom-val"),
        ]
    );
}

#[rstest]
fn round_trip_preserves_metadata_block(#[with(103)] line_shape: Shape) {
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    let metadata = "\n<gauss:doc-version>2</gauss:doc-version>";
    let svg = export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: Some(metadata),
        web_ready: false,
    });
    let imported = import_ok(&svg);
    // Export normalises content to end with newline before </metadata>,
    // so the imported block includes the trailing newline.
    assert_eq!(
        imported.gauss_metadata_block.as_deref(),
        Some("\n<gauss:doc-version>2</gauss:doc-version>\n")
    );
}
