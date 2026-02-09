//! Tests for SVG export output.

use super::*;
use crate::model::{
    Anchor, Gradient, GradientId, GradientKind, GradientStop, LinearGradient, PaintStyle, PathGeom,
    PatternId, PatternResource, Rgba, Shape, SymbolResource, Vec2,
};
use crate::test_helpers::shape_id_from_seed as shape_id;
use rstest::rstest;

fn test_gradient_sunset(resources: &mut ResourceStore) -> GradientId {
    resources.insert_gradient(Gradient::new(
        "sunset",
        GradientKind::Linear(LinearGradient::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            vec![
                GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
            ],
        )),
    ))
}

fn test_pattern_dots(resources: &mut ResourceStore) -> PatternId {
    resources.insert_pattern(PatternResource::new("dots", "<circle />"))
}

fn create_test_triangle(seed: u32, style: PaintStyle) -> Shape {
    Shape {
        id: shape_id(seed.into()),
        z: 0,
        style,
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
    }
}

fn build_doc_with_shape(shape: Shape) -> Document {
    let mut doc = Document::new();
    doc.append_shape(shape);
    doc
}

#[rstest]
fn exports_empty_document_with_valid_svg_root() {
    let doc = Document::new();
    let svg = export_svg(&doc, 100.0, 50.0);
    assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
    assert!(svg.contains(r#"viewBox="0 0 100 50""#));
}

#[rstest]
fn exports_simple_line_path() {
    let shape = Shape {
        id: shape_id(1),
        z: 0,
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(1.0, 2.0)),
                Anchor::new(Vec2::new(3.0, 4.0)),
            ],
            segments: vec![SegmentKind::Line],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
    };

    let mut doc = Document::new();
    doc.append_shape(shape);
    let svg = export_svg(&doc, 10.0, 10.0);

    assert!(svg.contains(r#"d="M 1 2 L 3 4""#));
    assert!(svg.contains(r##"stroke="#000000""##));
    assert!(svg.contains(r#"stroke-width="1""#));
    assert!(svg.contains(r#"fill="none""#));
}

#[rstest]
fn exports_opacity_when_alpha_is_not_opaque() {
    let shape = Shape {
        id: shape_id(2),
        z: 0,
        style: PaintStyle::new(
            Some(Rgba::new(255, 0, 0, 128)),
            2.0,
            Some(Rgba::new(0, 0, 0, 64)),
        ),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(0.0, 0.0)),
                Anchor::new(Vec2::new(1.0, 1.0)),
            ],
            segments: vec![SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
    };

    let mut doc = Document::new();
    doc.append_shape(shape);
    let svg = export_svg(&doc, 10.0, 10.0);

    assert!(svg.contains(r#"stroke-opacity=""#));
    assert!(svg.contains(r#"fill-opacity=""#));
}

#[rstest]
fn exports_gradient_and_pattern_defs_and_references() {
    let mut resources = ResourceStore::new();
    let gradient_id = test_gradient_sunset(&mut resources);
    let pattern_id = test_pattern_dots(&mut resources);
    let shape = create_test_triangle(
        3,
        PaintStyle::new_with_paint(
            Paint::gradient(gradient_id),
            1.5,
            Paint::pattern(pattern_id),
        ),
    );
    let doc = build_doc_with_shape(shape);
    let svg = export_svg_with_resources(&doc, &resources, 10.0, 10.0);

    assert!(svg.contains("<defs>"));
    assert!(svg.contains("<linearGradient id=\"sunset\""));
    assert!(svg.contains("<pattern id=\"dots\""));
    assert!(svg.contains("stroke=\"url(#sunset)\""));
    assert!(svg.contains("fill=\"url(#dots)\""));
}

#[rstest]
fn exports_paint_server_opacity_attributes() {
    let mut resources = ResourceStore::new();
    let gradient_id = test_gradient_sunset(&mut resources);
    let pattern_id = test_pattern_dots(&mut resources);
    let shape = create_test_triangle(
        30,
        PaintStyle::new_with_paint(
            Paint::gradient(gradient_id).with_opacity(128),
            1.0,
            Paint::pattern(pattern_id).with_opacity(64),
        ),
    );
    let doc = build_doc_with_shape(shape);
    let svg = export_svg_with_resources(&doc, &resources, 10.0, 10.0);

    assert!(svg.contains("stroke=\"url(#sunset)\""));
    assert!(svg.contains("fill=\"url(#dots)\""));
    assert!(svg.contains("stroke-opacity=\"0.5020\""));
    assert!(svg.contains("fill-opacity=\"0.2510\""));
}

#[rstest]
fn exports_pattern_and_symbol_extra_attributes() {
    let mut resources = ResourceStore::new();
    let _pattern_id = resources.insert_pattern(PatternResource::new_with_attributes(
        "dots",
        "<circle />",
        vec![
            ("patternUnits".to_owned(), "userSpaceOnUse".to_owned()),
            ("patternTransform".to_owned(), "scale(2)".to_owned()),
        ],
    ));
    let _symbol_id = resources.insert_symbol(SymbolResource::new_with_attributes(
        "badge",
        Some("0 0 10 10".to_owned()),
        "<rect width=\"10\" height=\"10\" />",
        vec![("preserveAspectRatio".to_owned(), "xMidYMid".to_owned())],
    ));

    let svg = export_svg_with_resources(&Document::new(), &resources, 10.0, 10.0);
    assert!(svg.contains(
        "<pattern id=\"dots\" patternUnits=\"userSpaceOnUse\" patternTransform=\"scale(2)\">"
    ));
    assert!(
        svg.contains(
            "<symbol id=\"badge\" viewBox=\"0 0 10 10\" preserveAspectRatio=\"xMidYMid\">"
        )
    );
}

#[rstest]
fn checked_export_reports_missing_resource_references() {
    let mut resources = ResourceStore::new();
    let dangling_gradient = test_gradient_sunset(&mut resources);
    let _removed = resources.remove_gradient(dangling_gradient);
    let shape = create_test_triangle(
        31,
        PaintStyle::new_with_paint(Paint::gradient(dangling_gradient), 1.0, Paint::None),
    );
    let doc = build_doc_with_shape(shape);

    let exported = export_svg_with_resources_checked(&doc, &resources, 10.0, 10.0);
    assert_eq!(
        exported,
        Err(SvgExportError::MissingGradientReference(dangling_gradient))
    );
}
