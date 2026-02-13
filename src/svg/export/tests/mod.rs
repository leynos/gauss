//! Tests for SVG export output.

mod resource_export;

use super::*;
use crate::model::{
    Anchor, Gradient, GradientId, GradientKind, GradientStop, LinearGradient, PaintStyle, PathGeom,
    PatternId, PatternResource, Rgba, Shape, SymbolResource, Vec2,
};
use crate::svg::metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX};
use crate::test_helpers::shape_id_from_seed as shape_id;
use rstest::{fixture, rstest};

struct PathData(String);
struct CssColor(String);
struct CssLength(String);
struct ResourceRef(String);
struct CssOpacity(String);

macro_rules! newtype_str {
    ($name:ident) => {
        impl $name {
            fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

newtype_str!(PathData);
newtype_str!(CssColor);
newtype_str!(CssLength);
newtype_str!(ResourceRef);
newtype_str!(CssOpacity);

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
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
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
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Verify SVG root element structure (without namespace checks).
fn assert_svg_root_element(svg: &str, expected_width: f32, expected_height: f32) {
    let document =
        roxmltree::Document::parse(svg).expect("exported SVG should always be valid XML");
    let root = document.root_element();
    assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
    assert!(svg.contains(&format!(
        r#"viewBox="0 0 {expected_width} {expected_height}""#
    )));
    assert_eq!(root.tag_name().name(), "svg");
}

/// Verify Gauss metadata namespace declaration on root.
fn assert_gauss_namespace_declared(svg: &str) {
    let document =
        roxmltree::Document::parse(svg).expect("exported SVG should always be valid XML");
    let root = document.root_element();
    let metadata_namespace =
        format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#);
    assert_eq!(
        root.lookup_namespace_uri(Some(GAUSS_METADATA_PREFIX)),
        Some(GAUSS_METADATA_NAMESPACE)
    );
    assert_eq!(svg.matches(metadata_namespace.as_str()).count(), 1);
}

/// Combined assertion: verify complete SVG root structure.
fn assert_valid_svg_root(svg: &str, expected_width: f32, expected_height: f32) {
    assert_svg_root_element(svg, expected_width, expected_height);
    assert_gauss_namespace_declared(svg);
}

/// Expected path attributes for SVG path assertions.
type SvgPathExpectation = (PathData, CssColor, CssLength, CssColor);

/// Custom assertion: verify path with stroke and fill attributes.
fn assert_svg_path(svg: &str, expected: SvgPathExpectation) {
    let (d, stroke, stroke_width, fill) = expected;
    assert!(svg.contains(&format!(r#"d="{}""#, d.as_ref())));
    assert!(svg.contains(&format!(r#"stroke="{}""#, stroke.as_ref())));
    assert!(svg.contains(&format!(r#"stroke-width="{}""#, stroke_width.as_ref())));
    assert!(svg.contains(&format!(r#"fill="{}""#, fill.as_ref())));
}

/// Custom assertion: verify gradient and pattern definitions and references.
fn assert_gradient_pattern_defs(svg: &str) {
    const EXPECTED_GRADIENT_ID: &str = "sunset";
    const EXPECTED_PATTERN_ID: &str = "dots";
    assert!(svg.contains("<defs>"));
    assert!(svg.contains(&format!(r#"<linearGradient id="{EXPECTED_GRADIENT_ID}""#)));
    assert!(svg.contains(&format!(r#"<pattern id="{EXPECTED_PATTERN_ID}""#)));
    assert!(svg.contains(&format!(r#"stroke="url(#{EXPECTED_GRADIENT_ID})""#)));
    assert!(svg.contains(&format!(r#"fill="url(#{EXPECTED_PATTERN_ID})""#)));
}

/// Expected paint server opacity attributes for SVG assertions.
type PaintServerOpacityExpectation = (ResourceRef, ResourceRef, CssOpacity, CssOpacity);

/// Custom assertion: verify paint server opacity attributes.
fn assert_paint_server_opacity(svg: &str, expected: PaintServerOpacityExpectation) {
    let (gradient_id, pattern_id, stroke_opacity, fill_opacity) = expected;
    assert!(svg.contains(&format!(r#"stroke="url(#{})""#, gradient_id.as_ref())));
    assert!(svg.contains(&format!(r#"fill="url(#{})""#, pattern_id.as_ref())));
    assert!(svg.contains(&format!(r#"stroke-opacity="{}""#, stroke_opacity.as_ref())));
    assert!(svg.contains(&format!(r#"fill-opacity="{}""#, fill_opacity.as_ref())));
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
    assert_svg_path(
        &svg,
        (
            PathData::new("M 1 2 L 3 4"),
            CssColor::new("#000000"),
            CssLength::new("1"),
            CssColor::new("none"),
        ),
    );
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
