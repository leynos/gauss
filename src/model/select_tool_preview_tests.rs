//! Preview/restore edge tests for `SelectTool` drag helpers.

use super::{
    Document, EdgeMode, SelectAnchorHit, SelectDragDocumentSnapshot, SelectHandleHit,
    SelectHandleHitKind, SelectPointerDownInput, SelectPointerHit, SelectShapeHit, SelectTool,
    SelectToolState, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, Vec2,
    apply_select_drag_preview, restore_select_drag_preview,
};
use rstest::rstest;

use super::select_tool_test_helpers::{selection_for_shape, shape_id, shape_with_handles};

fn extract_drag_state(commands: &[ToolCommand]) -> SelectToolState {
    commands
        .iter()
        .find_map(|command| match command {
            ToolCommand::SetSelectToolState(state) => Some(state.clone()),
            _ => None,
        })
        .unwrap_or(SelectToolState::Idle)
}

struct StalePreviewCase<'a> {
    shape_id: ShapeId,
    hit: SelectPointerHit,
    cursor_world: Vec2,
    previous_selection: super::Selection,
    preview_offset: Vec2,
    apply_error_msg: &'a str,
    restore_error_msg: &'a str,
}

fn test_stale_preview_returns_false<F>(case: StalePreviewCase<'_>, make_stale: F)
where
    F: FnOnce(&mut Document),
{
    let mut doc = Document::new();
    let _new_shape = doc.append_shape(shape_with_handles(case.shape_id));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(SelectPointerDownInput {
                drag_snapshot: SelectDragDocumentSnapshot::from_document(&doc),
                previous_selection: case.previous_selection,
                hit: case.hit,
                cursor_world: case.cursor_world,
                is_shift_held: false,
            }),
        },
    );

    let state = extract_drag_state(&down.commands);
    assert!(matches!(state, SelectToolState::Dragging(_)));

    make_stale(&mut doc);

    assert!(
        !apply_select_drag_preview(&mut doc, &state, case.cursor_world.add(case.preview_offset)),
        "{}",
        case.apply_error_msg
    );
    assert!(
        !restore_select_drag_preview(&mut doc, &state),
        "{}",
        case.restore_error_msg
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_drag_shape_is_stale() {
    let dragged_shape = shape_id(501);
    test_stale_preview_returns_false(
        StalePreviewCase {
            shape_id: dragged_shape,
            hit: SelectPointerHit::Shape(SelectShapeHit {
                shape_index: 0,
                shape_id: dragged_shape,
            }),
            cursor_world: Vec2::new(3.0, 3.0),
            previous_selection: selection_for_shape(dragged_shape),
            preview_offset: Vec2::new(2.0, 3.0),
            apply_error_msg: "stale drag target should not preview into a mismatched document",
            restore_error_msg: "stale drag target should not restore into a mismatched document",
        },
        |doc| {
            let _removed = doc.remove_shape(0);
            let _replacement = doc.append_shape(shape_with_handles(shape_id(999)));
        },
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_anchor_snapshot_is_stale() {
    let dragged_shape = shape_id(502);
    test_stale_preview_returns_false(
        StalePreviewCase {
            shape_id: dragged_shape,
            hit: SelectPointerHit::Anchor(SelectAnchorHit {
                shape_index: 0,
                shape_id: dragged_shape,
                anchor_index: 3,
            }),
            cursor_world: Vec2::new(0.0, 12.0),
            previous_selection: super::Selection::empty(),
            preview_offset: Vec2::new(2.0, 1.0),
            apply_error_msg: "stale anchor index should not update preview",
            restore_error_msg: "stale anchor index should not restore preview",
        },
        |doc| {
            let Some(shape) = doc.shape_at_mut(0) else {
                panic!("shape 0 should exist")
            };
            let _removed = shape.path.anchors.pop();
        },
    );
}

#[rstest]
fn apply_and_restore_select_drag_preview_return_false_when_handle_snapshot_is_stale() {
    let dragged_shape = shape_id(503);
    test_stale_preview_returns_false(
        StalePreviewCase {
            shape_id: dragged_shape,
            hit: SelectPointerHit::Handle(SelectHandleHit {
                shape_index: 0,
                shape_id: dragged_shape,
                anchor_index: 0,
                kind: SelectHandleHitKind::Out,
            }),
            cursor_world: Vec2::new(2.0, 1.0),
            previous_selection: super::Selection::empty(),
            preview_offset: Vec2::new(5.0, 3.0),
            apply_error_msg: "removed shape should prevent handle preview updates",
            restore_error_msg: "removed shape should prevent handle preview restore",
        },
        |doc| {
            let _removed = doc.remove_shape(0);
        },
    );
}
