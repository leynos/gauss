//! Unit tests for `SelectTool` manipulate transitions.

use super::{
    Anchor, Command, Document, EdgeMode, Paint, PaintStyle, PathGeom, SegmentKind,
    SelectPointerDownInput, SelectPointerHit, SelectPointerMoveInput, SelectPointerUpInput,
    SelectShapeHit, SelectTool, SelectToolState, Shape, ShapeId, Tool, ToolCommand, ToolInputEvent,
    ToolMode, Vec2,
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

fn square_shape(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
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

fn selection_for_shape(shape_id: ShapeId) -> super::Selection {
    super::Selection {
        items: vec![super::SelItem::Shape(shape_id)],
    }
}

fn pointer_down_input(
    doc: &Document,
    shape_id: ShapeId,
    cursor_world: Vec2,
    previous_selection: super::Selection,
) -> SelectPointerDownInput {
    SelectPointerDownInput {
        document: doc.clone(),
        previous_selection,
        hit: SelectPointerHit::Shape(SelectShapeHit {
            shape_index: 0,
            shape_id,
        }),
        cursor_world,
        is_shift_held: false,
    }
}

fn pointer_move_input(
    state: SelectToolState,
    cursor: Vec2,
    has_primary: bool,
) -> SelectPointerMoveInput {
    SelectPointerMoveInput {
        state,
        cursor_world: cursor,
        has_primary_button: has_primary,
    }
}

fn pointer_up_input(
    state: SelectToolState,
    cursor: Vec2,
    is_primary: bool,
) -> SelectPointerUpInput {
    SelectPointerUpInput {
        state,
        cursor_world: cursor,
        is_primary_button: is_primary,
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

fn setup_doc_with_shape(id: u64, min: Vec2, max: Vec2) -> (Document, ShapeId) {
    let mut doc = Document::new();
    let selected_shape = shape_id(id);
    let _new_shape = doc.append_shape(square_shape(selected_shape, min, max));
    (doc, selected_shape)
}

fn initiate_drag(doc: &Document, shape_id: ShapeId, cursor_pos: Vec2) -> SelectToolState {
    let down_commands = perform_transition(
        ToolMode::Manipulate,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                doc,
                shape_id,
                cursor_pos,
                selection_for_shape(shape_id),
            )),
        },
    );
    extract_drag_state(&down_commands)
}

fn perform_transition(mode: ToolMode, event: ToolInputEvent) -> Vec<ToolCommand> {
    Tool::transition(&SelectTool, mode, EdgeMode::Line, event).commands
}

#[rstest]
fn select_tool_pointer_down_selects_shape_and_enters_dragging_state() {
    let mut doc = Document::new();
    let selected_shape = shape_id(41);
    let _new_shape = doc.append_shape(square_shape(
        selected_shape,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));

    let select_transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                selected_shape,
                Vec2::new(5.0, 5.0),
                super::Selection::empty(),
            )),
        },
    );

    assert!(matches!(
        select_transition.commands.first(),
        Some(ToolCommand::RecordSelectionChange { .. })
    ));
    assert!(matches!(
        select_transition.commands.get(1),
        Some(ToolCommand::SetSelection(selection))
            if *selection == selection_for_shape(selected_shape)
    ));
    assert!(matches!(
        select_transition.commands.last(),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));

    let drag_transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(pointer_down_input(
                &doc,
                selected_shape,
                Vec2::new(5.0, 5.0),
                selection_for_shape(selected_shape),
            )),
        },
    );

    assert!(matches!(
        drag_transition.commands.last(),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Dragging(
            _
        )))
    ));
}

#[rstest]
fn select_tool_shift_click_toggles_selection_and_stays_idle() {
    let mut doc = Document::new();
    let selected_shape = shape_id(52);
    let _new_shape = doc.append_shape(square_shape(
        selected_shape,
        Vec2::new(0.0, 0.0),
        Vec2::new(12.0, 12.0),
    ));

    let transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::SelectPointerDown {
            input: Box::new(SelectPointerDownInput {
                document: doc,
                previous_selection: super::Selection::empty(),
                hit: SelectPointerHit::Shape(SelectShapeHit {
                    shape_index: 0,
                    shape_id: selected_shape,
                }),
                cursor_world: Vec2::new(6.0, 6.0),
                is_shift_held: true,
            }),
        },
    );

    assert!(matches!(
        transition.commands.last(),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));
}

#[rstest]
fn select_tool_pointer_move_with_drag_state_emits_preview_command() {
    let (doc, selected_shape) =
        setup_doc_with_shape(77, Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0));
    let drag_state = initiate_drag(&doc, selected_shape, Vec2::new(5.0, 5.0));
    assert!(matches!(drag_state, SelectToolState::Dragging(_)));

    let move_commands = perform_transition(
        ToolMode::Manipulate,
        ToolInputEvent::SelectPointerMove {
            input: Box::new(pointer_move_input(drag_state, Vec2::new(8.0, 9.0), true)),
        },
    );

    assert_eq!(
        move_commands,
        vec![ToolCommand::PreviewSelectDrag {
            cursor_world: Vec2::new(8.0, 9.0),
        }]
    );
}

#[rstest]
fn select_tool_pointer_move_without_drag_state_is_noop() {
    let transition = perform_transition(
        ToolMode::Manipulate,
        ToolInputEvent::SelectPointerMove {
            input: Box::new(pointer_move_input(
                SelectToolState::Idle,
                Vec2::new(8.0, 9.0),
                true,
            )),
        },
    );

    assert!(transition.is_empty());
}

#[rstest]
fn select_tool_pointer_up_without_delta_restores_preview_and_returns_idle() {
    let (doc, selected_shape) =
        setup_doc_with_shape(88, Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0));
    let drag_state = initiate_drag(&doc, selected_shape, Vec2::new(5.0, 5.0));
    assert!(matches!(drag_state, SelectToolState::Dragging(_)));
    let up_commands = perform_transition(
        ToolMode::Manipulate,
        ToolInputEvent::SelectPointerUp {
            input: Box::new(pointer_up_input(drag_state, Vec2::new(5.0, 5.0), true)),
        },
    );

    assert_eq!(
        up_commands,
        vec![
            ToolCommand::RestoreSelectDragPreview,
            ToolCommand::SetSelectToolState(SelectToolState::Idle),
        ]
    );
}

#[rstest]
fn select_tool_pointer_up_with_delta_emits_document_command_and_returns_idle() {
    let (doc, selected_shape) =
        setup_doc_with_shape(99, Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0));
    let drag_state = initiate_drag(&doc, selected_shape, Vec2::new(5.0, 5.0));
    assert!(matches!(drag_state, SelectToolState::Dragging(_)));
    let up_commands = perform_transition(
        ToolMode::Manipulate,
        ToolInputEvent::SelectPointerUp {
            input: Box::new(pointer_up_input(drag_state, Vec2::new(11.0, 7.0), true)),
        },
    );

    assert!(matches!(
        up_commands.first(),
        Some(ToolCommand::RestoreSelectDragPreview)
    ));
    assert!(matches!(
        up_commands.get(1),
        Some(ToolCommand::ApplyDocumentCommand(command))
            if matches!(command.as_ref(), Command::MoveShapes { .. })
    ));
    assert!(matches!(
        up_commands.last(),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));
}

#[rstest]
fn select_tool_ignores_pointer_events_outside_manipulate_mode() {
    let transition = perform_transition(
        ToolMode::Draw,
        ToolInputEvent::SelectPointerMove {
            input: Box::new(pointer_move_input(
                SelectToolState::Idle,
                Vec2::new(0.0, 0.0),
                true,
            )),
        },
    );

    assert!(transition.is_empty());
}
