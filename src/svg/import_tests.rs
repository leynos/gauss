//! Tests for SVG import.
//!
//! These tests validate that we can import a small SVG subset and that
//! Gauss-exported SVG round-trips through the importer.

use crate::{
    model::{Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2},
    svg::{export::export_svg, import::import_svg},
};
use rstest::rstest;
use uuid::Uuid;

#[rstest]
fn imports_minimal_line_path() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;

    let doc = import_svg(svg).expect("Expected import to succeed");
    assert_eq!(doc.shapes.len(), 1);

    let shape = doc
        .shapes
        .first()
        .expect("Expected imported SVG to contain a shape");
    assert_eq!(shape.path.anchors.len(), 2);
    assert_eq!(shape.path.segments, vec![SegmentKind::Line]);
    assert!(!shape.path.closed);
}

#[rstest]
fn round_trips_exported_svg() {
    let shape = Shape {
        id: ShapeId::from(Uuid::from_u128(1)),
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
        },
    };
    let doc = Document {
        shapes: vec![shape],
    };

    let exported = export_svg(&doc, 100.0, 100.0);
    let mut imported = import_svg(&exported).expect("Expected re-import to succeed");

    if let Some(imported_shape_mut) = imported.shapes.first_mut() {
        imported_shape_mut.id = ShapeId::from(Uuid::from_u128(1));
    }

    let imported_shape = imported
        .shapes
        .first()
        .expect("Expected re-imported SVG to contain a shape");
    assert!(
        (imported_shape.style.stroke_width - 1.0).abs() < 0.000_1,
        "Stroke width should round-trip through SVG"
    );
    assert!(imported_shape.path.closed);
}
