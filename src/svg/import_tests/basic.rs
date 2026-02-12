//! Basic geometry import tests for SVG path data.

use super::*;

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
#[expect(
    clippy::float_arithmetic,
    reason = "test compares round-tripped stroke width with tolerance"
)]
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
