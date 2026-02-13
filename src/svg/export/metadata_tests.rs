//! Tests for Gauss metadata export on `<path>` elements.

use slotmap::Key;

use super::*;
use crate::model::{Anchor, GaussAttribute, PaintStyle, PathGeom, Rgba, Shape, Vec2};
use crate::test_helpers::shape_id_from_seed as shape_id;
use rstest::rstest;

fn line_shape(seed: u32) -> Shape {
    Shape {
        id: shape_id(seed.into()),
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

fn export_single_shape(shape: Shape) -> String {
    let mut doc = Document::new();
    doc.append_shape(shape);
    export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_width: 100.0,
        canvas_height: 100.0,
        metadata_block: None,
    })
}

#[rstest]
fn exports_gauss_id_for_non_null_shape() {
    let shape = line_shape(1);
    let svg = export_single_shape(shape);
    assert!(svg.contains(r#"gauss:id="ffffffff00000001""#));
}

#[rstest]
fn omits_gauss_id_for_null_shape() {
    let mut shape = line_shape(1);
    shape.id = crate::model::ShapeId::null();
    let mut out = String::new();
    write_shape_gauss_metadata(&mut out, &shape);
    assert!(!out.contains("gauss:id"));
}

#[rstest]
fn exports_shape_name() {
    let mut shape = line_shape(2);
    shape.name = Some("My Shape".to_owned());
    let svg = export_single_shape(shape);
    assert!(svg.contains(r#"gauss:name="My Shape""#));
}

#[rstest]
fn omits_shape_name_when_none() {
    let shape = line_shape(3);
    let svg = export_single_shape(shape);
    assert!(!svg.contains("gauss:name"));
}

#[rstest]
fn exports_locked_attribute() {
    let mut shape = line_shape(4);
    shape.locked = true;
    let svg = export_single_shape(shape);
    assert!(svg.contains(r#"gauss:locked="true""#));
}

#[rstest]
fn omits_locked_attribute_when_false() {
    let shape = line_shape(5);
    let svg = export_single_shape(shape);
    assert!(!svg.contains("gauss:locked"));
}

#[rstest]
fn exports_hidden_attribute() {
    let mut shape = line_shape(6);
    shape.hidden = true;
    let svg = export_single_shape(shape);
    assert!(svg.contains(r#"gauss:hidden="true""#));
}

#[rstest]
fn exports_opaque_gauss_attrs() {
    let mut shape = line_shape(7);
    shape.gauss_metadata = vec![
        GaussAttribute::new("layer", "foreground"),
        GaussAttribute::new("opacity", "0.5"),
    ];
    let svg = export_single_shape(shape);
    assert!(svg.contains(r#"gauss:layer="foreground""#));
    assert!(svg.contains(r#"gauss:opacity="0.5""#));
}

#[rstest]
fn exports_metadata_block_when_present() {
    let doc = Document::new();
    let svg = export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_width: 100.0,
        canvas_height: 100.0,
        metadata_block: Some("\n<gauss:doc>test</gauss:doc>"),
    });
    assert!(svg.contains("<metadata>"));
    assert!(svg.contains("<gauss:doc>test</gauss:doc>"));
    assert!(svg.contains("</metadata>"));
}

#[rstest]
fn metadata_block_with_trailing_newline_does_not_add_extra_newline() {
    let doc = Document::new();
    let svg = export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_width: 100.0,
        canvas_height: 100.0,
        metadata_block: Some("test\n"),
    });
    assert!(svg.contains("<metadata>test\n</metadata>"));
    // Verify exactly one newline before </metadata>, not two.
    assert!(!svg.contains("test\n\n</metadata>"));
}

#[rstest]
fn omits_metadata_block_when_absent() {
    let doc = Document::new();
    let svg = export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_width: 100.0,
        canvas_height: 100.0,
        metadata_block: None,
    });
    assert!(!svg.contains("<metadata>"));
}
