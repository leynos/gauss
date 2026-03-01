//! Preview/restore edge tests for `SelectTool` drag helpers.

use super::{
    Anchor, Document, EdgeMode, Paint, PaintStyle, PathGeom, SegmentKind, SelectAnchorHit,
    SelectHandleHit, SelectHandleHitKind, SelectPointerDownInput, SelectPointerHit, SelectShapeHit,
    SelectTool, SelectToolState, Shape, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, Vec2,
    apply_select_drag_preview, restore_select_drag_preview,
};
use rstest::rstest;

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn default_style() -> PaintStyle {
    PaintStyle {
        stroke: Paint::Solid(super::Rgba::new(16, 32, 64, 255)),
        stroke_width: 2.0,
        fill: Paint::None,
    }
}

fn shape_with_handles(id: ShapeId) -> Shape {
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

fn selection_for_shape(shape_id: ShapeId) -> super::Selection {
    super::Selection {
        items: vec![super::SelItem::Shape(shape_id)],
    }
}

fn extract_drag_state(commands: &[ToolCommand]) -> SelectToolState {
    commands
        .iter()
        .find_map(|command| match command {
            ToolCommand::SetSelectToolState(state) => Some(state.clone()),
            _ => None,
        })
        .unwrap_or(SelectToolState::Idle)
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_drag_shape_is_stale() {
    let mut doc = Document::new();
    let dragged_shape = shape_id(501);
    let _new_shape = doc.append_shape(shape_with_handles(dragged_shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(SelectPointerDownInput {
                document: doc.clone(),
                previous_selection: selection_for_shape(dragged_shape),
                hit: SelectPointerHit::Shape(SelectShapeHit {
                    shape_index: 0,
                    shape_id: dragged_shape,
                }),
                cursor_world: Vec2::new(3.0, 3.0),
                is_shift_held: false,
            }),
        },
    );

    let state = extract_drag_state(&down.commands);
    assert!(matches!(state, SelectToolState::Dragging(_)));

    let _removed = doc.remove_shape(0);
    let _replacement = doc.append_shape(shape_with_handles(shape_id(999)));

    assert!(
        !apply_select_drag_preview(&mut doc, &state, Vec2::new(5.0, 6.0)),
        "stale drag target should not preview into a mismatched document"
    );
    assert!(
        !restore_select_drag_preview(&mut doc, &state),
        "stale drag target should not restore into a mismatched document"
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_anchor_snapshot_is_stale() {
    let mut doc = Document::new();
    let dragged_shape = shape_id(502);
    let _new_shape = doc.append_shape(shape_with_handles(dragged_shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(SelectPointerDownInput {
                document: doc.clone(),
                previous_selection: super::Selection::empty(),
                hit: SelectPointerHit::Anchor(SelectAnchorHit {
                    shape_index: 0,
                    shape_id: dragged_shape,
                    anchor_index: 3,
                }),
                cursor_world: Vec2::new(0.0, 12.0),
                is_shift_held: false,
            }),
        },
    );

    let state = extract_drag_state(&down.commands);
    assert!(matches!(state, SelectToolState::Dragging(_)));

    let Some(shape) = doc.shape_at_mut(0) else {
        panic!("shape 0 should exist")
    };
    let _removed = shape.path.anchors.pop();

    assert!(
        !apply_select_drag_preview(&mut doc, &state, Vec2::new(2.0, 13.0)),
        "stale anchor index should not update preview"
    );
    assert!(
        !restore_select_drag_preview(&mut doc, &state),
        "stale anchor index should not restore preview"
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_handle_snapshot_is_stale() {
    let mut doc = Document::new();
    let dragged_shape = shape_id(503);
    let _new_shape = doc.append_shape(shape_with_handles(dragged_shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(SelectPointerDownInput {
                document: doc.clone(),
                previous_selection: super::Selection::empty(),
                hit: SelectPointerHit::Handle(SelectHandleHit {
                    shape_index: 0,
                    shape_id: dragged_shape,
                    anchor_index: 0,
                    kind: SelectHandleHitKind::Out,
                }),
                cursor_world: Vec2::new(2.0, 1.0),
                is_shift_held: false,
            }),
        },
    );

    let state = extract_drag_state(&down.commands);
    assert!(matches!(state, SelectToolState::Dragging(_)));

    let _removed = doc.remove_shape(0);

    assert!(
        !apply_select_drag_preview(&mut doc, &state, Vec2::new(7.0, 4.0)),
        "removed shape should prevent handle preview updates"
    );
    assert!(
        !restore_select_drag_preview(&mut doc, &state),
        "removed shape should prevent handle preview restore"
    );
}
