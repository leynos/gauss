//! Tests for Gauss metadata export on `<path>` elements.

use super::*;
use crate::model::{
    Anchor, GaussAttribute, Gradient, GradientKind, GradientStop, LinearGradient, Paint,
    PaintStyle, PathGeom, PatternResource, Rgba, Shape, Vec2,
};
use crate::test_helpers::shape_id_from_seed as shape_id;
use rstest::{fixture, rstest};

#[fixture]
fn line_shape(#[default(1)] seed: u32) -> Shape {
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

#[fixture]
fn export_single_shape(line_shape: Shape) -> String {
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: None,
        web_ready: false,
    })
}

#[rstest]
fn exports_gauss_id_for_non_null_shape(export_single_shape: String) {
    assert!(export_single_shape.contains(r#"gauss:id="ffffffff00000001""#));
}

#[rstest]
fn omits_gauss_id_for_null_shape(mut line_shape: Shape) {
    line_shape.id = crate::model::ShapeId::null();
    let mut out = String::new();
    write_shape_gauss_metadata(&mut out, &line_shape);
    assert!(!out.contains("gauss:id"));
}

#[rstest]
fn exports_shape_name(#[with(2)] mut line_shape: Shape) {
    line_shape.name = Some("My Shape".to_owned());
    let svg = export_single_shape(line_shape);
    assert!(svg.contains(r#"gauss:name="My Shape""#));
}

#[rstest]
fn omits_shape_name_when_none(export_single_shape: String) {
    assert!(!export_single_shape.contains("gauss:name"));
}

#[rstest]
fn exports_locked_attribute(#[with(4)] mut line_shape: Shape) {
    line_shape.locked = true;
    let svg = export_single_shape(line_shape);
    assert!(svg.contains(r#"gauss:locked="true""#));
}

#[rstest]
fn omits_locked_attribute_when_false(export_single_shape: String) {
    assert!(!export_single_shape.contains("gauss:locked"));
}

#[rstest]
fn exports_hidden_attribute(#[with(6)] mut line_shape: Shape) {
    line_shape.hidden = true;
    let svg = export_single_shape(line_shape);
    assert!(svg.contains(r#"gauss:hidden="true""#));
}

#[rstest]
fn exports_opaque_gauss_attrs(#[with(7)] mut line_shape: Shape) {
    line_shape.gauss_metadata = vec![
        GaussAttribute::new("layer", "foreground"),
        GaussAttribute::new("opacity", "0.5"),
    ];
    let svg = export_single_shape(line_shape);
    assert!(svg.contains(r#"gauss:layer="foreground""#));
    assert!(svg.contains(r#"gauss:opacity="0.5""#));
}

#[rstest]
fn exports_metadata_block_when_present() {
    let doc = Document::new();
    let svg = export_svg_with_metadata(ExportOptions {
        doc: &doc,
        resources: &ResourceStore::new(),
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: Some("\n<gauss:doc>test</gauss:doc>"),
        web_ready: false,
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
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: Some("test\n"),
        web_ready: false,
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
        canvas_size: CanvasSize::new(100.0, 100.0),
        metadata_block: None,
        web_ready: false,
    });
    assert!(!svg.contains("<metadata>"));
}

#[rstest]
fn web_ready_export_strips_gauss_shape_metadata(#[with(20)] mut line_shape: Shape) {
    line_shape.name = Some("Web Ready".to_owned());
    line_shape.locked = true;
    line_shape.hidden = true;
    line_shape.gauss_metadata = vec![GaussAttribute::new("role", "overlay")];
    let mut doc = Document::new();
    doc.append_shape(line_shape);
    let svg = export_svg_with_resources_web_ready(
        &doc,
        &ResourceStore::new(),
        CanvasSize::new(100.0, 100.0),
    );
    assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
    assert!(svg.contains("<path "));
    assert!(!svg.contains("xmlns:gauss="));
    assert!(!svg.contains(" gauss:"));
    assert!(!svg.contains("<metadata>"));
}

#[rstest]
fn web_ready_export_keeps_valid_svg_shell_for_empty_document() {
    let svg = export_svg_with_resources_web_ready(
        &Document::new(),
        &ResourceStore::new(),
        CanvasSize::new(120.0, 80.0),
    );
    assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"));
    assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80""#));
    assert!(svg.ends_with("</svg>\n"));
}

#[rstest]
fn web_ready_checked_export_reports_missing_gradient_reference(#[with(30)] line_shape: Shape) {
    let mut resources = ResourceStore::new();
    let dangling_gradient = resources.insert_gradient(Gradient::new(
        "sunset",
        GradientKind::Linear(LinearGradient::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            vec![
                GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                GradientStop::new(1.0, Rgba::new(0, 0, 255, 255)),
            ],
        )),
    ));
    let _removed = resources.remove_gradient(dangling_gradient);
    let mut shape = line_shape;
    shape.style = PaintStyle::new_with_paint(Paint::gradient(dangling_gradient), 1.0, Paint::None);
    let mut doc = Document::new();
    doc.append_shape(shape);
    let exported = export_svg_with_resources_web_ready_checked(
        &doc,
        &resources,
        CanvasSize::new(100.0, 100.0),
    );
    assert_eq!(
        exported,
        Err(SvgExportError::MissingGradientReference(dangling_gradient))
    );
}

#[rstest]
fn web_ready_checked_export_reports_missing_pattern_reference(#[with(31)] line_shape: Shape) {
    let mut resources = ResourceStore::new();
    let dangling_pattern = resources.insert_pattern(PatternResource::new(
        "dots",
        r#"<circle cx="1" cy="1" r="1" />"#,
    ));
    let _removed = resources.remove_pattern(dangling_pattern);
    let mut shape = line_shape;
    shape.style = PaintStyle::new_with_paint(Paint::None, 1.0, Paint::pattern(dangling_pattern));
    let mut doc = Document::new();
    doc.append_shape(shape);
    let exported = export_svg_with_resources_web_ready_checked(
        &doc,
        &resources,
        CanvasSize::new(100.0, 100.0),
    );
    assert_eq!(
        exported,
        Err(SvgExportError::MissingPatternReference(dangling_pattern))
    );
}
