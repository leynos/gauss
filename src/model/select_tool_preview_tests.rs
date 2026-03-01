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

#[expect(
    clippy::too_many_arguments,
    reason = "test helper intentionally accepts explicit case parameters"
)]
fn test_stale_preview_returns_false<F>(
    shape_id: ShapeId,
    hit: SelectPointerHit,
    cursor_world: Vec2,
    previous_selection: super::Selection,
    make_stale: F,
    preview_offset: Vec2,
    apply_error_msg: &str,
    restore_error_msg: &str,
) where
    F: FnOnce(&mut Document),
{
    let mut doc = Document::new();
    let _new_shape = doc.append_shape(shape_with_handles(shape_id));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(SelectPointerDownInput {
                document: doc.clone(),
                previous_selection,
                hit,
                cursor_world,
                is_shift_held: false,
            }),
        },
    );

    let state = extract_drag_state(&down.commands);
    assert!(matches!(state, SelectToolState::Dragging(_)));

    make_stale(&mut doc);

    assert!(
        !apply_select_drag_preview(&mut doc, &state, cursor_world.add(preview_offset)),
        "{apply_error_msg}"
    );
    assert!(
        !restore_select_drag_preview(&mut doc, &state),
        "{restore_error_msg}"
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_drag_shape_is_stale() {
    let dragged_shape = shape_id(501);
    test_stale_preview_returns_false(
        dragged_shape,
        SelectPointerHit::Shape(SelectShapeHit {
            shape_index: 0,
            shape_id: dragged_shape,
        }),
        Vec2::new(3.0, 3.0),
        selection_for_shape(dragged_shape),
        |doc| {
            let _removed = doc.remove_shape(0);
            let _replacement = doc.append_shape(shape_with_handles(shape_id(999)));
        },
        Vec2::new(2.0, 3.0),
        "stale drag target should not preview into a mismatched document",
        "stale drag target should not restore into a mismatched document",
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_anchor_snapshot_is_stale() {
    let dragged_shape = shape_id(502);
    test_stale_preview_returns_false(
        dragged_shape,
        SelectPointerHit::Anchor(SelectAnchorHit {
            shape_index: 0,
            shape_id: dragged_shape,
            anchor_index: 3,
        }),
        Vec2::new(0.0, 12.0),
        super::Selection::empty(),
        |doc| {
            let Some(shape) = doc.shape_at_mut(0) else {
                panic!("shape 0 should exist")
            };
            let _removed = shape.path.anchors.pop();
        },
        Vec2::new(2.0, 1.0),
        "stale anchor index should not update preview",
        "stale anchor index should not restore preview",
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_handle_snapshot_is_stale() {
    let dragged_shape = shape_id(503);
    test_stale_preview_returns_false(
        dragged_shape,
        SelectPointerHit::Handle(SelectHandleHit {
            shape_index: 0,
            shape_id: dragged_shape,
            anchor_index: 0,
            kind: SelectHandleHitKind::Out,
        }),
        Vec2::new(2.0, 1.0),
        super::Selection::empty(),
        |doc| {
            let _removed = doc.remove_shape(0);
        },
        Vec2::new(5.0, 3.0),
        "removed shape should prevent handle preview updates",
        "removed shape should prevent handle preview restore",
    );
}
