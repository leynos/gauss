//! Additional unit tests for `SelectTool` drag-state transitions.

use super::{
    Anchor, Command, Document, EdgeMode, Paint, PaintStyle, PathGeom, SelectAnchorHit,
    SelectHandleHit, SelectHandleHitKind, SelectPointerDownInput, SelectPointerHit,
    SelectPointerMoveInput, SelectPointerUpInput, SelectSegmentHit, SelectShapeHit, SelectTool,
    SelectToolState, SegmentKind, Shape, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode,
    Vec2,
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

fn pointer_down_input(
    doc: &Document,
    hit: SelectPointerHit,
    cursor_world: Vec2,
    previous_selection: super::Selection,
) -> SelectPointerDownInput {
    SelectPointerDownInput {
        document: doc.clone(),
        previous_selection,
        hit,
        cursor_world,
        is_shift_held: false,
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
#[case(SelectToolState::Marquee)]
#[case(SelectToolState::Transforming)]
fn select_tool_pointer_move_is_noop_for_reserved_states(#[case] state: SelectToolState) {
    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerMove {
            input: Box::new(SelectPointerMoveInput {
                state,
                cursor_world: Vec2::new(3.0, 4.0),
                has_primary_button: true,
            }),
        },
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
#[case(SelectToolState::Marquee)]
#[case(SelectToolState::Transforming)]
fn select_tool_pointer_up_is_noop_for_reserved_states(#[case] state: SelectToolState) {
    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerUp {
            input: Box::new(SelectPointerUpInput {
                state,
                cursor_world: Vec2::new(3.0, 4.0),
                is_primary_button: true,
            }),
        },
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn select_tool_pointer_move_is_noop_when_dragging_without_primary_button() {
    let mut doc = Document::new();
    let selected_shape = shape_id(111);
    let _new_shape = doc.append_shape(shape_with_handles(selected_shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                SelectPointerHit::Shape(SelectShapeHit {
                    shape_index: 0,
                    shape_id: selected_shape,
                }),
                Vec2::new(5.0, 5.0),
                selection_for_shape(selected_shape),
            )),
        },
    );
    let drag_state = extract_drag_state(&down.commands);
    assert!(matches!(drag_state, SelectToolState::Dragging(_)));

    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerMove {
            input: Box::new(SelectPointerMoveInput {
                state: drag_state,
                cursor_world: Vec2::new(7.0, 8.0),
                has_primary_button: false,
            }),
        },
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn select_tool_pointer_up_is_noop_when_dragging_with_non_primary_button() {
    let mut doc = Document::new();
    let selected_shape = shape_id(112);
    let _new_shape = doc.append_shape(shape_with_handles(selected_shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                SelectPointerHit::Shape(SelectShapeHit {
                    shape_index: 0,
                    shape_id: selected_shape,
                }),
                Vec2::new(5.0, 5.0),
                selection_for_shape(selected_shape),
            )),
        },
    );
    let drag_state = extract_drag_state(&down.commands);
    assert!(matches!(drag_state, SelectToolState::Dragging(_)));

    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerUp {
            input: Box::new(SelectPointerUpInput {
                state: drag_state,
                cursor_world: Vec2::new(7.0, 8.0),
                is_primary_button: false,
            }),
        },
    );

    assert!(transition.commands.is_empty());
}

enum ExpectedDragKind {
    Anchor,
    Handle,
    Shapes,
}

fn assert_drag_kind(commands: &[ToolCommand], expected: ExpectedDragKind) {
    let state = commands.iter().find_map(|command| match command {
        ToolCommand::SetSelectToolState(state) => Some(state),
        _ => None,
    });

    match (state, expected) {
        (
            Some(SelectToolState::Dragging(super::select_tool::SelectDragState::Anchor(_))),
            ExpectedDragKind::Anchor,
        )
        | (
            Some(SelectToolState::Dragging(super::select_tool::SelectDragState::Handle(_))),
            ExpectedDragKind::Handle,
        )
        | (
            Some(SelectToolState::Dragging(super::select_tool::SelectDragState::Shapes(_))),
            ExpectedDragKind::Shapes,
        ) => {}
        _ => panic!("unexpected select tool drag state"),
    }
}

#[rstest]
#[case(
    SelectPointerHit::Anchor(SelectAnchorHit {
        shape_index: 0,
        shape_id: shape_id(201),
        anchor_index: 0,
    }),
    ExpectedDragKind::Anchor
)]
#[case(
    SelectPointerHit::Handle(SelectHandleHit {
        shape_index: 0,
        shape_id: shape_id(201),
        anchor_index: 0,
        kind: SelectHandleHitKind::Out,
    }),
    ExpectedDragKind::Handle
)]
#[case(
    SelectPointerHit::Segment(SelectSegmentHit {
        shape_index: 0,
        shape_id: shape_id(201),
        seg_index: 0,
    }),
    ExpectedDragKind::Shapes
)]
fn select_tool_pointer_down_enters_expected_drag_state(
    #[case] hit: SelectPointerHit,
    #[case] expected: ExpectedDragKind,
) {
    let mut doc = Document::new();
    let shape = shape_id(201);
    let _new_shape = doc.append_shape(shape_with_handles(shape));

    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                hit,
                Vec2::new(1.0, 1.0),
                super::Selection::empty(),
            )),
        },
    );

    assert_drag_kind(&transition.commands, expected);
}

#[rstest]
fn select_tool_pointer_up_after_anchor_drag_emits_move_anchor_and_idle() {
    let mut doc = Document::new();
    let shape = shape_id(301);
    let _new_shape = doc.append_shape(shape_with_handles(shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                SelectPointerHit::Anchor(SelectAnchorHit {
                    shape_index: 0,
                    shape_id: shape,
                    anchor_index: 0,
                }),
                Vec2::new(2.0, 2.0),
                super::Selection::empty(),
            )),
        },
    );

    let drag_state = extract_drag_state(&down.commands);
    let up = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerUp {
            input: Box::new(SelectPointerUpInput {
                state: drag_state,
                cursor_world: Vec2::new(6.0, 5.0),
                is_primary_button: true,
            }),
        },
    );

    assert!(matches!(
        up.commands.first(),
        Some(ToolCommand::RestoreSelectDragPreview)
    ));
    assert!(matches!(
        up.commands.get(1),
        Some(ToolCommand::ApplyDocumentCommand(command))
            if matches!(command.as_ref(), Command::MoveAnchor { .. })
    ));
    assert!(matches!(
        up.commands.last(),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));
}

#[rstest]
fn select_tool_pointer_up_after_handle_drag_emits_move_handle_and_idle() {
    let mut doc = Document::new();
    let shape = shape_id(302);
    let _new_shape = doc.append_shape(shape_with_handles(shape));

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                SelectPointerHit::Handle(SelectHandleHit {
                    shape_index: 0,
                    shape_id: shape,
                    anchor_index: 0,
                    kind: SelectHandleHitKind::Out,
                }),
                Vec2::new(2.0, 2.0),
                super::Selection::empty(),
            )),
        },
    );

    let drag_state = extract_drag_state(&down.commands);
    let up = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerUp {
            input: Box::new(SelectPointerUpInput {
                state: drag_state,
                cursor_world: Vec2::new(6.0, 5.0),
                is_primary_button: true,
            }),
        },
    );

    assert!(matches!(
        up.commands.first(),
        Some(ToolCommand::RestoreSelectDragPreview)
    ));
    assert!(matches!(
        up.commands.get(1),
        Some(ToolCommand::ApplyDocumentCommand(command))
            if matches!(command.as_ref(), Command::MoveHandle { .. })
    ));
    assert!(matches!(
        up.commands.last(),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));
}
