//! Tests for SVG export output.

use super::*;
use crate::model::{
    Anchor, Gradient, GradientId, GradientKind, GradientStop, LinearGradient, PaintStyle, PathGeom,
    PatternId, PatternResource, RadialGradient, Rgba, Shape, SymbolResource, Vec2,
};
use crate::svg::metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX};
use crate::test_helpers::shape_id_from_seed as shape_id;
use rstest::{fixture, rstest};

#[fixture]
fn test_gradient_sunset() -> impl Fn(&mut ResourceStore) -> GradientId {
    |resources| {
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
}

#[fixture]
fn test_pattern_dots() -> impl Fn(&mut ResourceStore) -> PatternId {
    |resources| resources.insert_pattern(PatternResource::new("dots", "<circle />"))
}

#[fixture]
fn create_test_triangle() -> impl Fn(u32, PaintStyle) -> Shape {
    |seed, style| Shape {
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

#[fixture]
fn build_doc_with_shape() -> impl Fn(Shape) -> Document {
    |shape| {
        let mut doc = Document::new();
        doc.append_shape(shape);
        doc
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Test setup helper centralises shared fixture wiring for two cases."
)]
fn setup_gradient_pattern_export<
    TestGradientSunset,
    TestPatternDots,
    CreateTestTriangle,
    BuildDocWithShape,
>(
    seed: u32,
    stroke_width: f32,
    stroke_opacity: Option<u8>,
    fill_opacity: Option<u8>,
    test_gradient_sunset: &TestGradientSunset,
    test_pattern_dots: &TestPatternDots,
    create_test_triangle: &CreateTestTriangle,
    build_doc_with_shape: &BuildDocWithShape,
) -> (Document, ResourceStore)
where
    TestGradientSunset: Fn(&mut ResourceStore) -> GradientId,
    TestPatternDots: Fn(&mut ResourceStore) -> PatternId,
    CreateTestTriangle: Fn(u32, PaintStyle) -> Shape,
    BuildDocWithShape: Fn(Shape) -> Document,
{
    let mut resources = ResourceStore::new();
    let gradient_id = test_gradient_sunset(&mut resources);
    let pattern_id = test_pattern_dots(&mut resources);

    let mut stroke = Paint::gradient(gradient_id);
    if let Some(opacity) = stroke_opacity {
        stroke = stroke.with_opacity(opacity);
    }

    let mut fill = Paint::pattern(pattern_id);
    if let Some(opacity) = fill_opacity {
        fill = fill.with_opacity(opacity);
    }

    let shape = create_test_triangle(seed, PaintStyle::new_with_paint(stroke, stroke_width, fill));
    let doc = build_doc_with_shape(shape);

    (doc, resources)
}

fn create_line_shape_for_export(
    seed: u32,
    line: (Vec2, Vec2),
    style: PaintStyle,
    closed: bool,
) -> Shape {
    let (start, end) = line;
    Shape {
        id: shape_id(seed.into()),
        z: 0,
        style,
        path: PathGeom {
            anchors: vec![Anchor::new(start), Anchor::new(end)],
            segments: vec![SegmentKind::Line],
            closed,
            closing_segment: SegmentKind::Line,
        },
    }
}

/// Custom assertion: verify SVG root element structure.
fn assert_valid_svg_root(svg: &str, expected_width: f32, expected_height: f32) {
    let document =
        roxmltree::Document::parse(svg).expect("exported SVG should always be valid XML");
    let root = document.root_element();
    let metadata_namespace =
        format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#);

    assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
    assert!(svg.contains(&format!(
        r#"viewBox="0 0 {expected_width} {expected_height}""#
    )));
    assert_eq!(root.tag_name().name(), "svg");
    assert_eq!(
        root.lookup_namespace_uri(Some(GAUSS_METADATA_PREFIX)),
        Some(GAUSS_METADATA_NAMESPACE)
    );
    assert_eq!(svg.matches(metadata_namespace.as_str()).count(), 1);
}

/// Expected path attributes for SVG path assertions.
type SvgPathExpectation<'a> = (&'a str, &'a str, &'a str, &'a str);

/// Custom assertion: verify path with stroke and fill attributes.
fn assert_svg_path(svg: &str, expected: SvgPathExpectation<'_>) {
    let (d, stroke, stroke_width, fill) = expected;
    assert!(svg.contains(&format!(r#"d="{d}""#)));
    assert!(svg.contains(&format!(r#"stroke="{stroke}""#)));
    assert!(svg.contains(&format!(r#"stroke-width="{stroke_width}""#)));
    assert!(svg.contains(&format!(r#"fill="{fill}""#)));
}

/// Custom assertion: verify gradient and pattern definitions and references.
fn assert_gradient_pattern_defs(svg: &str, gradient_id: &str, pattern_id: &str) {
    assert!(svg.contains("<defs>"));
    assert!(svg.contains(&format!(r#"<linearGradient id="{gradient_id}""#)));
    assert!(svg.contains(&format!(r#"<pattern id="{pattern_id}""#)));
    assert!(svg.contains(&format!(r#"stroke="url(#{gradient_id})""#)));
    assert!(svg.contains(&format!(r#"fill="url(#{pattern_id})""#)));
}

/// Expected paint server opacity attributes for SVG assertions.
type PaintServerOpacityExpectation<'a> = (&'a str, &'a str, &'a str, &'a str);

/// Custom assertion: verify paint server opacity attributes.
fn assert_paint_server_opacity(svg: &str, expected: PaintServerOpacityExpectation<'_>) {
    let (gradient_id, pattern_id, stroke_opacity, fill_opacity) = expected;
    assert!(svg.contains(&format!(r#"stroke="url(#{gradient_id})""#)));
    assert!(svg.contains(&format!(r#"fill="url(#{pattern_id})""#)));
    assert!(svg.contains(&format!(r#"stroke-opacity="{stroke_opacity}""#)));
    assert!(svg.contains(&format!(r#"fill-opacity="{fill_opacity}""#)));
}

#[rstest]
fn exports_empty_document_with_valid_svg_root() {
    let doc = Document::new();
    let svg = export_svg(&doc, 100.0, 50.0);
    assert_valid_svg_root(&svg, 100.0, 50.0);
}

#[rstest]
fn exports_simple_line_path(build_doc_with_shape: impl Fn(Shape) -> Document) {
    let shape = create_line_shape_for_export(
        1,
        (Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)),
        PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 1.0, None),
        false,
    );

    let doc = build_doc_with_shape(shape);
    let svg = export_svg(&doc, 10.0, 10.0);

    assert_svg_path(&svg, ("M 1 2 L 3 4", "#000000", "1", "none"));
}

#[rstest]
fn exports_opacity_when_alpha_is_not_opaque(build_doc_with_shape: impl Fn(Shape) -> Document) {
    let shape = create_line_shape_for_export(
        2,
        (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        PaintStyle::new(
            Some(Rgba::new(255, 0, 0, 128)),
            2.0,
            Some(Rgba::new(0, 0, 0, 64)),
        ),
        true,
    );

    let doc = build_doc_with_shape(shape);
    let svg = export_svg(&doc, 10.0, 10.0);

    assert!(svg.contains(r#"stroke-opacity="0.5020""#));
    assert!(svg.contains(r#"fill-opacity="0.2510""#));
}

#[rstest]
fn exports_gradient_and_pattern_defs_and_references(
    test_gradient_sunset: impl Fn(&mut ResourceStore) -> GradientId,
    test_pattern_dots: impl Fn(&mut ResourceStore) -> PatternId,
    create_test_triangle: impl Fn(u32, PaintStyle) -> Shape,
    build_doc_with_shape: impl Fn(Shape) -> Document,
) {
    let (doc, resources) = setup_gradient_pattern_export(
        3,
        1.5,
        None,
        None,
        &test_gradient_sunset,
        &test_pattern_dots,
        &create_test_triangle,
        &build_doc_with_shape,
    );
    let svg = export_svg_with_resources(&doc, &resources, 10.0, 10.0);

    assert_gradient_pattern_defs(&svg, "sunset", "dots");
}

#[rstest]
fn exports_paint_server_opacity_attributes(
    test_gradient_sunset: impl Fn(&mut ResourceStore) -> GradientId,
    test_pattern_dots: impl Fn(&mut ResourceStore) -> PatternId,
    create_test_triangle: impl Fn(u32, PaintStyle) -> Shape,
    build_doc_with_shape: impl Fn(Shape) -> Document,
) {
    let (doc, resources) = setup_gradient_pattern_export(
        30,
        1.0,
        Some(128),
        Some(64),
        &test_gradient_sunset,
        &test_pattern_dots,
        &create_test_triangle,
        &build_doc_with_shape,
    );
    let svg = export_svg_with_resources(&doc, &resources, 10.0, 10.0);

    assert_paint_server_opacity(&svg, ("sunset", "dots", "0.5020", "0.2510"));
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
fn exports_radial_gradient_defs_with_and_without_focal_point() {
    let mut resources = ResourceStore::new();
    let _first = resources.insert_gradient(Gradient::new(
        "radial-default",
        GradientKind::Radial(RadialGradient::new(
            Vec2::new(0.5, 0.5),
            0.4,
            None,
            vec![
                GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                GradientStop::new(1.0, Rgba::new(0, 0, 255, 255)),
            ],
        )),
    ));
    let _second = resources.insert_gradient(Gradient::new(
        "radial-focal",
        GradientKind::Radial(RadialGradient::new(
            Vec2::new(0.5, 0.5),
            0.4,
            Some(Vec2::new(0.3, 0.2)),
            vec![
                GradientStop::new(0.0, Rgba::new(255, 255, 255, 255)),
                GradientStop::new(1.0, Rgba::new(0, 0, 0, 255)),
            ],
        )),
    ));

    let svg = export_svg_with_resources(&Document::new(), &resources, 10.0, 10.0);
    assert!(svg.contains(r#"<radialGradient id="radial-default" cx="0.5" cy="0.5" r="0.4">"#));
    assert!(svg.contains(
        r#"<radialGradient id="radial-focal" cx="0.5" cy="0.5" r="0.4" fx="0.3" fy="0.2">"#
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Helper keeps fixture and resource constructors explicit for the paired error tests"
)]
fn assert_missing_resource_error<ResourceId, F, PaintFn, ErrorFn, CreateTriangleFn, BuildDocFn>(
    create_and_remove: F,
    seed: u32,
    make_paint: PaintFn,
    expected_error: ErrorFn,
    create_test_triangle: CreateTriangleFn,
    build_doc_with_shape: BuildDocFn,
) where
    F: FnOnce(&mut ResourceStore) -> ResourceId,
    PaintFn: Fn(ResourceId) -> PaintStyle,
    ErrorFn: Fn(ResourceId) -> SvgExportError,
    CreateTriangleFn: Fn(u32, PaintStyle) -> Shape,
    BuildDocFn: Fn(Shape) -> Document,
    ResourceId: Copy,
{
    let mut resources = ResourceStore::new();
    let dangling_id = create_and_remove(&mut resources);
    let shape = create_test_triangle(seed, make_paint(dangling_id));
    let doc = build_doc_with_shape(shape);

    let exported = export_svg_with_resources_checked(&doc, &resources, 10.0, 10.0);
    assert_eq!(exported, Err(expected_error(dangling_id)));
}

#[rstest]
fn checked_export_reports_missing_resource_references(
    test_gradient_sunset: impl Fn(&mut ResourceStore) -> GradientId,
    create_test_triangle: impl Fn(u32, PaintStyle) -> Shape,
    build_doc_with_shape: impl Fn(Shape) -> Document,
) {
    assert_missing_resource_error(
        |resources| {
            let dangling_gradient = test_gradient_sunset(resources);
            let _removed = resources.remove_gradient(dangling_gradient);
            dangling_gradient
        },
        31,
        |id| PaintStyle::new_with_paint(Paint::gradient(id), 1.0, Paint::None),
        SvgExportError::MissingGradientReference,
        create_test_triangle,
        build_doc_with_shape,
    );
}

#[rstest]
fn checked_export_reports_missing_pattern_references(
    test_pattern_dots: impl Fn(&mut ResourceStore) -> PatternId,
    create_test_triangle: impl Fn(u32, PaintStyle) -> Shape,
    build_doc_with_shape: impl Fn(Shape) -> Document,
) {
    assert_missing_resource_error(
        |resources| {
            let dangling_pattern = test_pattern_dots(resources);
            let _removed = resources.remove_pattern(dangling_pattern);
            dangling_pattern
        },
        32,
        |id| PaintStyle::new_with_paint(Paint::None, 1.0, Paint::pattern(id)),
        SvgExportError::MissingPatternReference,
        create_test_triangle,
        build_doc_with_shape,
    );
}
