//! Additional unit tests for `SelectTool` drag-state transitions.

use super::{
    Anchor, Command, Document, EdgeMode, Paint, PaintStyle, PathGeom, SegmentKind, SelectAnchorHit,
    SelectHandleHit, SelectHandleHitKind, SelectPointerDownInput, SelectPointerHit,
    SelectPointerMoveInput, SelectPointerUpInput, SelectSegmentHit, SelectShapeHit, SelectTool,
    SelectToolState, Shape, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, Vec2,
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
    let extracted_state = commands.iter().find_map(|command| match command {
        ToolCommand::SetSelectToolState(state) => Some(state.clone()),
        _ => None,
    });
    let Some(state) = extracted_state else {
        panic!("expected ToolCommand::SetSelectToolState emission for SelectToolState");
    };
    state
}

fn setup_drag_test(
    shape_id: ShapeId,
    hit: SelectPointerHit,
    cursor_pos: Vec2,
) -> (Document, SelectToolState) {
    let mut doc = Document::new();
    let _new_shape = doc.append_shape(shape_with_handles(shape_id));
    let previous_selection = match &hit {
        SelectPointerHit::Shape(_) | SelectPointerHit::Segment(_) => selection_for_shape(shape_id),
        SelectPointerHit::Anchor(_) | SelectPointerHit::Handle(_) | SelectPointerHit::None => {
            super::Selection::empty()
        }
    };

    let down = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                hit,
                cursor_pos,
                previous_selection,
            )),
        },
    );

    (doc, extract_drag_state(&down.commands))
}

fn assert_pointer_event_is_noop(state: SelectToolState, event: ToolInputEvent) {
    let normalized_event = match event {
        ToolInputEvent::SelectPointerMove { input } => ToolInputEvent::SelectPointerMove {
            input: Box::new(SelectPointerMoveInput {
                is_dragging: matches!(state, SelectToolState::Dragging(_)),
                cursor_world: input.cursor_world,
                has_primary_button: input.has_primary_button,
            }),
        },
        ToolInputEvent::SelectPointerUp { input } => ToolInputEvent::SelectPointerUp {
            input: Box::new(SelectPointerUpInput {
                state,
                cursor_world: input.cursor_world,
                is_primary_button: input.is_primary_button,
            }),
        },
        _ => panic!("assert_pointer_event_is_noop requires pointer move or pointer up"),
    };

    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        normalized_event,
    );
    assert!(transition.commands.is_empty());
}

fn pointer_move_without_primary_event() -> ToolInputEvent {
    ToolInputEvent::SelectPointerMove {
        input: Box::new(SelectPointerMoveInput {
            is_dragging: false,
            cursor_world: Vec2::new(7.0, 8.0),
            has_primary_button: false,
        }),
    }
}

fn pointer_up_without_primary_event() -> ToolInputEvent {
    ToolInputEvent::SelectPointerUp {
        input: Box::new(SelectPointerUpInput {
            state: SelectToolState::Idle,
            cursor_world: Vec2::new(7.0, 8.0),
            is_primary_button: false,
        }),
    }
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
                is_dragging: matches!(state, SelectToolState::Dragging(_)),
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
#[case(
    shape_id(111),
    SelectPointerHit::Shape(SelectShapeHit {
        shape_index: 0,
        shape_id: shape_id(111),
    }),
    pointer_move_without_primary_event
)]
#[case(
    shape_id(112),
    SelectPointerHit::Shape(SelectShapeHit {
        shape_index: 0,
        shape_id: shape_id(112),
    }),
    pointer_up_without_primary_event
)]
fn select_tool_pointer_events_are_noop_when_dragging_without_primary_button(
    #[case] selected_shape_id: ShapeId,
    #[case] hit: SelectPointerHit,
    #[case] event_factory: fn() -> ToolInputEvent,
) {
    let (_doc, drag_state) = setup_drag_test(selected_shape_id, hit, Vec2::new(5.0, 5.0));
    assert!(matches!(drag_state, SelectToolState::Dragging(_)));

    assert_pointer_event_is_noop(drag_state, event_factory());
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

enum ExpectedPointerUpCommand {
    MoveAnchor,
    MoveHandle,
}

#[rstest]
#[case(
    shape_id(301),
    SelectPointerHit::Anchor(SelectAnchorHit {
        shape_index: 0,
        shape_id: shape_id(301),
        anchor_index: 0,
    }),
    ExpectedPointerUpCommand::MoveAnchor
)]
#[case(
    shape_id(302),
    SelectPointerHit::Handle(SelectHandleHit {
        shape_index: 0,
        shape_id: shape_id(302),
        anchor_index: 0,
        kind: SelectHandleHitKind::Out,
    }),
    ExpectedPointerUpCommand::MoveHandle
)]
fn select_tool_pointer_up_after_control_point_drag_emits_expected_command_and_idle(
    #[case] selected_shape_id: ShapeId,
    #[case] hit: SelectPointerHit,
    #[case] expected: ExpectedPointerUpCommand,
) {
    let (_doc, drag_state) = setup_drag_test(selected_shape_id, hit, Vec2::new(2.0, 2.0));
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
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));
    match expected {
        ExpectedPointerUpCommand::MoveAnchor => assert!(matches!(
            up.commands.get(2),
            Some(ToolCommand::ApplyDocumentCommand(command))
                if matches!(command.as_ref(), Command::MoveAnchor { .. })
        )),
        ExpectedPointerUpCommand::MoveHandle => assert!(matches!(
            up.commands.get(2),
            Some(ToolCommand::ApplyDocumentCommand(command))
                if matches!(command.as_ref(), Command::MoveHandle { .. })
        )),
    }
}
