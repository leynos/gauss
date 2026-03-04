//! Unit tests for `SelectTool` manipulate transitions.

use super::{
    Command, Document, EdgeMode, SelectDragDocumentSnapshot, SelectPointerDownInput,
    SelectPointerHit, SelectPointerMoveInput, SelectPointerUpInput, SelectShapeHit, SelectTool,
    SelectToolState, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, Vec2,
};
use rstest::rstest;

use super::select_tool_test_helpers::{selection_for_shape, shape_id, square_shape};

fn pointer_down_input(
    doc: &Document,
    shape_id: ShapeId,
    cursor_world: Vec2,
    previous_selection: super::Selection,
) -> SelectPointerDownInput {
    SelectPointerDownInput {
        drag_snapshot: SelectDragDocumentSnapshot::from_document(doc),
        previous_selection,
        hit: SelectPointerHit::Shape(SelectShapeHit {
            shape_index: 0,
            shape_id,
        }),
        cursor_world,
        is_shift_held: false,
    }
}

#[derive(Clone, Copy)]
enum PointerInputKind {
    Move,
    Up,
}

fn pointer_input(
    state: SelectToolState,
    cursor: Vec2,
    is_primary: bool,
    kind: PointerInputKind,
) -> ToolInputEvent {
    let is_dragging = matches!(&state, SelectToolState::Dragging(_));
    match kind {
        PointerInputKind::Move => ToolInputEvent::SelectPointerMove {
            input: Box::new(SelectPointerMoveInput {
                is_dragging,
                cursor_world: cursor,
                has_primary_button: is_primary,
            }),
        },
        PointerInputKind::Up => ToolInputEvent::SelectPointerUp {
            input: Box::new(SelectPointerUpInput {
                state,
                cursor_world: cursor,
                is_primary_button: is_primary,
            }),
        },
    }
}

fn extract_drag_state(commands: &[ToolCommand]) -> SelectToolState {
    let extracted_state = commands.iter().find_map(|command| match command {
        ToolCommand::SetSelectToolState(state) => Some(state.clone()),
        _ => None,
    });
    let Some(state) = extracted_state else {
        panic!("expected SetSelectToolState command to be emitted");
    };
    state
}

struct SelectToolTestFixture {
    document: Document,
    shape_id: ShapeId,
    edge_mode: EdgeMode,
}

impl SelectToolTestFixture {
    fn new(id: u64, size: f32) -> Self {
        let mut document = Document::new();
        let selected_shape = shape_id(id);
        let _new_shape = document.append_shape(square_shape(
            selected_shape,
            Vec2::new(0.0, 0.0),
            Vec2::new(size, size),
        ));

        Self {
            document,
            shape_id: selected_shape,
            edge_mode: EdgeMode::Line,
        }
    }

    fn transition_pointer_down(
        &self,
        cursor: Vec2,
        previous_selection: super::Selection,
    ) -> super::ToolTransition {
        self.transition_with_mode(
            ToolMode::Manipulate,
            ToolInputEvent::SelectPointerDown {
                input: Box::new(pointer_down_input(
                    &self.document,
                    self.shape_id,
                    cursor,
                    previous_selection,
                )),
            },
        )
    }

    fn transition_with_mode(&self, mode: ToolMode, event: ToolInputEvent) -> super::ToolTransition {
        Tool::transition(&SelectTool, mode, self.edge_mode, event)
    }

    /// Initiates a drag and returns the drag state, asserting it's valid.
    fn setup_drag_state(&self, cursor: Vec2, selection: super::Selection) -> SelectToolState {
        let down = self.transition_pointer_down(cursor, selection);
        let drag_state = extract_drag_state(&down.commands);
        assert!(matches!(drag_state, SelectToolState::Dragging(_)));
        drag_state
    }
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
                drag_snapshot: SelectDragDocumentSnapshot::from_document(&doc),
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
    let fixture = SelectToolTestFixture::new(77, 20.0);
    let drag_state =
        fixture.setup_drag_state(Vec2::new(5.0, 5.0), selection_for_shape(shape_id(77)));

    let move_transition = fixture.transition_with_mode(
        ToolMode::Manipulate,
        pointer_input(
            drag_state,
            Vec2::new(8.0, 9.0),
            true,
            PointerInputKind::Move,
        ),
    );

    assert_eq!(
        move_transition.commands,
        vec![ToolCommand::PreviewSelectDrag {
            cursor_world: Vec2::new(8.0, 9.0),
        }]
    );
}

#[rstest]
#[case::without_drag_state(1, ToolMode::Manipulate, SelectToolState::Idle, Vec2::new(8.0, 9.0))]
#[case::outside_manipulate_mode(2, ToolMode::Draw, SelectToolState::Idle, Vec2::new(0.0, 0.0))]
fn select_tool_pointer_move_noop_scenarios(
    #[case] fixture_id: u64,
    #[case] mode: ToolMode,
    #[case] state: SelectToolState,
    #[case] cursor: Vec2,
) {
    let fixture = SelectToolTestFixture::new(fixture_id, 1.0);
    let transition = fixture.transition_with_mode(
        mode,
        pointer_input(state, cursor, true, PointerInputKind::Move),
    );

    assert!(transition.commands.is_empty());
}

#[rstest]
fn select_tool_pointer_up_without_delta_restores_preview_and_returns_idle() {
    let fixture = SelectToolTestFixture::new(88, 20.0);
    let drag_state =
        fixture.setup_drag_state(Vec2::new(5.0, 5.0), selection_for_shape(shape_id(88)));
    let up = fixture.transition_with_mode(
        ToolMode::Manipulate,
        pointer_input(drag_state, Vec2::new(5.0, 5.0), true, PointerInputKind::Up),
    );

    assert_eq!(
        up.commands,
        vec![
            ToolCommand::RestoreSelectDragPreview,
            ToolCommand::SetSelectToolState(SelectToolState::Idle),
        ]
    );
}

#[rstest]
fn select_tool_pointer_up_with_delta_emits_document_command_and_returns_idle() {
    let fixture = SelectToolTestFixture::new(99, 20.0);
    let drag_state =
        fixture.setup_drag_state(Vec2::new(5.0, 5.0), selection_for_shape(shape_id(99)));
    let up = fixture.transition_with_mode(
        ToolMode::Manipulate,
        pointer_input(drag_state, Vec2::new(11.0, 7.0), true, PointerInputKind::Up),
    );

    assert!(matches!(
        up.commands.first(),
        Some(ToolCommand::RestoreSelectDragPreview)
    ));
    assert!(matches!(
        up.commands.get(1),
        Some(ToolCommand::SetSelectToolState(SelectToolState::Idle))
    ));
    assert!(matches!(
        up.commands.get(2),
        Some(ToolCommand::ApplyDocumentCommand(command))
            if matches!(command.as_ref(), Command::MoveShapes { .. })
    ));
}
