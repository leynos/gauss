//! Resource-related export tests: gradients, patterns, paint server opacity,
//! and checked export validation.

use super::*;
use crate::model::{
    Gradient, GradientId, GradientKind, GradientStop, PaintStyle, PatternId, RadialGradient, Rgba,
    Shape, Vec2,
};
use rstest::rstest;

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
    assert_gradient_pattern_defs(&svg);
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
    assert_paint_server_opacity(
        &svg,
        (
            ResourceRef::new("sunset"),
            ResourceRef::new("dots"),
            CssOpacity::new("0.5020"),
            CssOpacity::new("0.2510"),
        ),
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
