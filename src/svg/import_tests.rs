//! Tests for SVG import.
//!
//! These tests validate that we can import a small SVG subset and that
//! Gauss-exported SVG round-trips through the importer.

#![expect(
    clippy::float_arithmetic,
    reason = "tests perform floating-point comparisons for imported values"
)]

use crate::{
    model::{
        Anchor, Document, Gradient, GradientKind, GradientStop, LinearGradient, Paint, PaintStyle,
        PathGeom, PatternResource, ResourceStore, Rgba, SegmentKind, Shape, ShapeId,
        SymbolResource, Vec2,
    },
    svg::{
        export::export_svg_with_resources,
        import::{SvgImportError, import_svg, import_svg_with_resources},
    },
};
use rstest::rstest;

#[rstest]
fn imports_minimal_line_path() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;

    let doc = match import_svg(svg) {
        Ok(doc) => doc,
        Err(err) => panic!("Expected import to succeed: {err}"),
    };
    assert_eq!(doc.len(), 1);

    let Some(shape) = doc.shape_at(0) else {
        panic!("Expected imported SVG to contain a shape");
    };
    assert_eq!(shape.path.anchors.len(), 2);
    assert_eq!(shape.path.segments, vec![SegmentKind::Line]);
    assert!(!shape.path.closed);
}

#[rstest]
fn round_trips_exported_svg() {
    let shape = Shape {
        id: ShapeId::default(),
        z: 0,
        style: PaintStyle::new(
            Some(Rgba::new(0, 0, 0, 255)),
            1.0,
            Some(Rgba::new(0, 0, 255, 128)),
        ),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(0.0, 0.0)),
                Anchor {
                    pos: Vec2::new(10.0, 0.0),
                    handle_in: Some(Vec2::new(7.0, 1.0)),
                    handle_out: None,
                },
            ],
            segments: vec![SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
    };
    let mut doc = Document::new();
    doc.append_shape(shape);

    let exported = export_svg_with_resources(&doc, &ResourceStore::new(), 100.0, 100.0);
    let imported = match import_svg(&exported) {
        Ok(imported_doc) => imported_doc,
        Err(err) => panic!("Expected re-import to succeed: {err}"),
    };

    let Some(imported_shape) = imported.shape_at(0) else {
        panic!("Expected re-imported SVG to contain a shape");
    };
    assert!(
        (imported_shape.style.stroke_width - 1.0).abs() < 0.000_1,
        "Stroke width should round-trip through SVG"
    );
    assert!(imported_shape.path.closed);
}

#[rstest]
fn imports_resource_defs_and_paint_refs() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="sunset" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0" stop-color="#ff0000" />
              <stop offset="1" stop-color="#ffff00" />
            </linearGradient>
            <pattern id="dots"><circle cx="1" cy="1" r="1" /></pattern>
          </defs>
          <path d="M 1 2 L 3 4 Z" stroke="url(#sunset)" stroke-opacity="0.5" stroke-width="2" fill="url(#dots)" fill-opacity="0.25" />
        </svg>
    "##;

    let imported = match import_svg_with_resources(svg) {
        Ok(imported) => imported,
        Err(err) => panic!("Expected resource import to succeed: {err}"),
    };
    assert_eq!(imported.resources.gradient_count(), 1);
    assert_eq!(imported.resources.pattern_count(), 1);

    let Some(gradient_id) = imported.resources.gradient_id_for_svg_id("sunset") else {
        panic!("Expected imported resources to include a sunset gradient ID");
    };
    let Some(pattern_id) = imported.resources.pattern_id_for_svg_id("dots") else {
        panic!("Expected imported resources to include a dots pattern ID");
    };

    let Some(shape) = imported.document.shape_at(0) else {
        panic!("Expected one imported shape");
    };
    assert_eq!(
        shape.style.stroke,
        Paint::gradient(gradient_id).with_opacity(128)
    );
    assert_eq!(
        shape.style.fill,
        Paint::pattern(pattern_id).with_opacity(64)
    );
}

#[rstest]
fn reports_missing_resource_refs() {
    let svg = r#"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="url(#missing)" stroke-width="1" fill="none" />
        </svg>
    "#;

    let result = import_svg_with_resources(svg);
    assert_eq!(
        result,
        Err(SvgImportError::MissingReferencedResource(
            "missing".to_owned()
        ))
    );
}

#[rstest]
fn imports_pattern_and_symbol_extra_attributes() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <defs>
            <pattern id="dots" patternUnits="userSpaceOnUse" patternTransform="scale(2)">
              <circle cx="1" cy="1" r="1" />
            </pattern>
            <symbol id="badge" viewBox="0 0 10 10" preserveAspectRatio="xMidYMid">
              <rect width="10" height="10" />
            </symbol>
          </defs>
          <path d="M 1 2 L 3 4 Z" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;

    let imported = match import_svg_with_resources(svg) {
        Ok(imported) => imported,
        Err(err) => panic!("Expected resource import to succeed: {err}"),
    };

    let Some(pattern_id) = imported.resources.pattern_id_for_svg_id("dots") else {
        panic!("Expected imported resources to include a dots pattern ID");
    };
    let Some(symbol_id) = imported.resources.symbol_id_for_svg_id("badge") else {
        panic!("Expected imported resources to include a badge symbol ID");
    };

    let Some(pattern) = imported.resources.pattern(pattern_id) else {
        panic!("Expected to retrieve imported pattern by ID");
    };
    assert_eq!(
        pattern.extra_attributes,
        vec![
            ("patternUnits".to_owned(), "userSpaceOnUse".to_owned()),
            ("patternTransform".to_owned(), "scale(2)".to_owned()),
        ]
    );

    let Some(symbol) = imported.resources.symbol(symbol_id) else {
        panic!("Expected to retrieve imported symbol by ID");
    };
    assert_eq!(symbol.view_box.as_deref(), Some("0 0 10 10"));
    assert_eq!(
        symbol.extra_attributes,
        vec![("preserveAspectRatio".to_owned(), "xMidYMid".to_owned())]
    );
}

#[rstest]
fn resource_round_trip_preserves_defs_and_references() {
    let mut resources = ResourceStore::new();
    let gradient_id = resources.insert_gradient(Gradient::new(
        "sunset",
        GradientKind::Linear(LinearGradient::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            vec![
                GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
            ],
        )),
    ));
    let pattern_id = resources.insert_pattern(PatternResource::new_with_attributes(
        "dots",
        "<circle />",
        vec![("patternUnits".to_owned(), "userSpaceOnUse".to_owned())],
    ));
    let _symbol_id = resources.insert_symbol(SymbolResource::new_with_attributes(
        "badge",
        Some("0 0 10 10".to_owned()),
        "<rect width=\"10\" height=\"10\" />",
        vec![("preserveAspectRatio".to_owned(), "xMidYMid".to_owned())],
    ));

    let shape = Shape {
        id: ShapeId::default(),
        z: 0,
        style: PaintStyle::new_with_paint(
            Paint::gradient(gradient_id),
            1.0,
            Paint::pattern(pattern_id),
        ),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(1.0, 2.0)),
                Anchor::new(Vec2::new(3.0, 4.0)),
            ],
            segments: vec![SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
    };

    let mut doc = Document::new();
    doc.append_shape(shape);

    let exported = export_svg_with_resources(&doc, &resources, 100.0, 100.0);
    let imported = match import_svg_with_resources(&exported) {
        Ok(imported) => imported,
        Err(err) => panic!("Expected resource round-trip import to succeed: {err}"),
    };
    assert_eq!(imported.resources.gradient_count(), 1);
    assert_eq!(imported.resources.pattern_count(), 1);
    assert_eq!(imported.resources.symbol_count(), 1);

    let Some(imported_pattern_id) = imported.resources.pattern_id_for_svg_id("dots") else {
        panic!("Expected imported resources to include a dots pattern ID");
    };
    let Some(imported_pattern) = imported.resources.pattern(imported_pattern_id) else {
        panic!("Expected imported pattern to be retrievable");
    };
    assert_eq!(
        imported_pattern.extra_attributes,
        vec![("patternUnits".to_owned(), "userSpaceOnUse".to_owned())]
    );

    let Some(imported_symbol_id) = imported.resources.symbol_id_for_svg_id("badge") else {
        panic!("Expected imported resources to include a badge symbol ID");
    };
    let Some(imported_symbol) = imported.resources.symbol(imported_symbol_id) else {
        panic!("Expected imported symbol to be retrievable");
    };
    assert_eq!(imported_symbol.view_box.as_deref(), Some("0 0 10 10"));
    assert_eq!(
        imported_symbol.extra_attributes,
        vec![("preserveAspectRatio".to_owned(), "xMidYMid".to_owned())]
    );

    let Some(imported_shape) = imported.document.shape_at(0) else {
        panic!("Expected one imported shape");
    };
    assert_eq!(imported_shape.style.stroke, Paint::gradient(gradient_id));
    assert_eq!(imported_shape.style.fill, Paint::pattern(pattern_id));
}
