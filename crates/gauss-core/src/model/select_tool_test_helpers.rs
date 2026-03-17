//! Shared helpers for `SelectTool` unit tests.
//!
//! Keeping these constructors in one place avoids drift across test modules
//! when shape fixtures or style defaults evolve.

use crate::model::{
    Anchor, Paint, PaintStyle, PathGeom, Rgba, SegmentKind, SelItem, Selection, Shape, ShapeId,
    Vec2,
};

pub(super) fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

pub(super) fn default_style() -> PaintStyle {
    PaintStyle {
        stroke: Paint::Solid(Rgba::new(16, 32, 64, 255)),
        stroke_width: 2.0,
        fill: Paint::None,
    }
}

pub(super) fn square_shape(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
    Shape {
        id,
        z: 0,
        style: default_style(),
        path: PathGeom {
            anchors: vec![
                Anchor::new(min),
                Anchor::new(Vec2::new(max.x, min.y)),
                Anchor::new(max),
                Anchor::new(Vec2::new(min.x, max.y)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

pub(super) fn shape_with_handles(id: ShapeId) -> Shape {
    Shape {
        id,
        z: 0,
        style: default_style(),
        path: PathGeom {
            anchors: vec![
                Anchor {
                    pos: Vec2::new(0.0, 0.0),
                    handle_in: Some(Vec2::new(-2.0, -1.0)),
                    handle_out: Some(Vec2::new(2.0, 1.0)),
                },
                Anchor::new(Vec2::new(12.0, 0.0)),
                Anchor::new(Vec2::new(12.0, 12.0)),
                Anchor::new(Vec2::new(0.0, 12.0)),
            ],
            segments: vec![SegmentKind::Cubic, SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

pub(super) fn selection_for_shape(shape_id: ShapeId) -> Selection {
    Selection {
        items: vec![SelItem::Shape(shape_id)],
    }
}
