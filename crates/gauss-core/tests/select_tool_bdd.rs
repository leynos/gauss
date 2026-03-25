//! Behaviour tests for `SelectTool` command emission.

use gauss_core::model::{
    Anchor, Document, EdgeMode, Paint, PaintStyle, PathGeom, Rgba, SegmentKind, SelectToolState,
    Shape, ShapeId, ToolInputEvent, ToolMode, ToolTransition, Vec2,
};
use rstest::fixture;

#[derive(Default)]
pub(crate) struct SelectToolWorld {
    pub(crate) mode: ToolMode,
    pub(crate) edge_mode: EdgeMode,
    pub(crate) input_event: Option<ToolInputEvent>,
    pub(crate) transition: Option<ToolTransition>,
    pub(crate) document: Document,
    pub(crate) shape_id: ShapeId,
    pub(crate) drag_state: Option<SelectToolState>,
}

pub(crate) fn make_shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

pub(crate) fn square_shape(id: ShapeId) -> Shape {
    let mut first_anchor = Anchor::new(Vec2::new(0.0, 0.0));
    first_anchor.handle_out = Some(Vec2::new(2.0, 0.0));

    Shape {
        id,
        z: 0,
        style: PaintStyle {
            stroke: Paint::Solid(Rgba::new(16, 32, 64, 255)),
            stroke_width: 2.0,
            fill: Paint::None,
        },
        path: PathGeom {
            anchors: vec![
                first_anchor,
                Anchor::new(Vec2::new(10.0, 0.0)),
                Anchor::new(Vec2::new(10.0, 10.0)),
                Anchor::new(Vec2::new(0.0, 10.0)),
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

#[fixture]
pub(crate) fn world() -> SelectToolWorld {
    let mut document = Document::new();
    let shape_id = make_shape_id(7);
    let _inserted = document.append_shape(square_shape(shape_id));

    SelectToolWorld {
        mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        input_event: None,
        transition: None,
        document,
        shape_id,
        drag_state: None,
    }
}

#[path = "select_tool_bdd/helpers.rs"]
mod helpers;
#[path = "select_tool_bdd/scenario_bindings.rs"]
mod scenario_bindings;
#[path = "select_tool_bdd/steps.rs"]
mod steps;
