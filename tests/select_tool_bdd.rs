//! Behaviour tests for `SelectTool` command emission.

use gauss::model::{
    Anchor, Command, Document, EdgeMode, Paint, PaintStyle, PathGeom, SegmentKind, SelectAnchorHit,
    SelectHandleHit, SelectHandleHitKind, SelectPointerDownInput, SelectPointerHit,
    SelectPointerMoveInput, SelectPointerUpInput, SelectSegmentHit, SelectShapeHit, SelectTool,
    SelectToolState, Shape, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolTransition,
    Vec2,
};
use rstest::fixture;
use rstest_bdd_macros::{given, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct SelectToolWorld {
    mode: ToolMode,
    edge_mode: EdgeMode,
    input_event: Option<ToolInputEvent>,
    transition: Option<ToolTransition>,
    document: Document,
    shape_id: ShapeId,
    drag_state: Option<SelectToolState>,
}

fn make_shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn square_shape(id: ShapeId) -> Shape {
    let mut first_anchor = Anchor::new(Vec2::new(0.0, 0.0));
    first_anchor.handle_out = Some(Vec2::new(2.0, 0.0));

    Shape {
        id,
        z: 0,
        style: PaintStyle {
            stroke: Paint::Solid(gauss::model::Rgba::new(16, 32, 64, 255)),
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

fn pointer_down_event(
    document: &Document,
    hit: SelectPointerHit,
    cursor_world: Vec2,
    is_shift_held: bool,
) -> ToolInputEvent {
    ToolInputEvent::SelectPointerDown {
        input: Box::new(SelectPointerDownInput {
            document: document.clone(),
            previous_selection: gauss::model::Selection::empty(),
            hit,
            cursor_world,
            is_shift_held,
        }),
    }
}

fn extract_drag_state(transition: &ToolTransition) -> Option<SelectToolState> {
    transition
        .commands
        .iter()
        .find_map(|command| match command {
            ToolCommand::SetSelectToolState(state) => Some(state.clone()),
            _ => None,
        })
}

fn transition(world: &SelectToolWorld) -> TestSupportResult<&ToolTransition> {
    world
        .transition
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("transition", "assertion"))
}

fn assert_contains_command(
    world: &SelectToolWorld,
    expected: &ToolCommand,
) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.iter().any(|actual| actual == expected) {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "Expected command {expected:?}; actual commands: {commands:?}"
    )))
}

fn assert_emits_command_matching<M>(
    world: &SelectToolWorld,
    matcher: M,
    expected_description: &str,
) -> TestSupportResult<()>
where
    M: Fn(&ToolCommand) -> bool,
{
    let commands = &transition(world)?.commands;
    if commands.iter().any(matcher) {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "expected {expected_description}; got {commands:?}"
    )))
}

fn set_drag_state_from_hit(
    world: &mut SelectToolWorld,
    hit: SelectPointerHit,
    cursor_world: Vec2,
) -> TestSupportResult<()> {
    let down_event = pointer_down_event(&world.document, hit, cursor_world, false);
    let down_transition = Tool::transition(
        &SelectTool,
        ToolMode::Manipulate,
        world.edge_mode,
        down_event,
    );
    world.drag_state = Some(extract_drag_state(&down_transition).ok_or_else(|| {
        TestSupportError::expectation(
            "expected pointer-down transition to yield SelectToolState".to_owned(),
        )
    })?);
    Ok(())
}

#[rustfmt::skip]
fn resolve_drag_setup(drag_kind: &str, shape_id: ShapeId) -> TestSupportResult<(SelectPointerHit, Vec2)> {
    match drag_kind {
        "shape" => Ok((SelectPointerHit::Segment(SelectSegmentHit { shape_index: 0, shape_id, seg_index: 0 }), Vec2::new(5.0, 5.0))),
        "anchor" => Ok((SelectPointerHit::Anchor(SelectAnchorHit { shape_index: 0, shape_id, anchor_index: 0 }), Vec2::new(0.0, 0.0))),
        "handle" => Ok((SelectPointerHit::Handle(SelectHandleHit { shape_index: 0, shape_id, anchor_index: 0, kind: SelectHandleHitKind::Out }), Vec2::new(2.0, 0.0))),
        _ => Err(TestSupportError::expectation(format!("unsupported drag kind '{drag_kind}'"))),
    }
}

fn resolve_state(world: &SelectToolWorld, state: &str) -> TestSupportResult<SelectToolState> {
    match state {
        "Idle" => Ok(SelectToolState::Idle),
        "Marquee" => Ok(SelectToolState::Marquee),
        "Transforming" => Ok(SelectToolState::Transforming),
        "Dragging" => world
            .drag_state
            .clone()
            .ok_or_else(|| TestSupportError::missing("drag_state", "Dragging state setup")),
        _ => Err(TestSupportError::expectation(format!(
            "unsupported state '{state}'"
        ))),
    }
}

#[rustfmt::skip]
fn resolve_up_position(position: &str) -> TestSupportResult<Vec2> {
    match position { "origin" => Ok(Vec2::new(5.0, 5.0)), "moved" => Ok(Vec2::new(11.0, 7.0)), _ => Err(TestSupportError::expectation(format!("unsupported pointer-up position '{position}'"))) }
}

#[fixture]
fn world() -> SelectToolWorld {
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

#[given("the select tool mode is Manipulate")]
fn given_mode_manipulate(world: &mut SelectToolWorld) {
    world.mode = ToolMode::Manipulate;
}

#[given("the select tool mode is Draw")]
fn given_mode_draw(world: &mut SelectToolWorld) {
    world.mode = ToolMode::Draw;
}

#[given("the select tool event is pointer down on shape without shift")]
#[rustfmt::skip]
fn given_pointer_down_without_shift(world: &mut SelectToolWorld) { world.input_event = Some(pointer_down_event(&world.document, SelectPointerHit::Segment(SelectSegmentHit { shape_index: 0, shape_id: world.shape_id, seg_index: 0 }), Vec2::new(5.0, 5.0), false)); }

#[given("the select tool event is pointer down on shape with shift")]
#[rustfmt::skip]
fn given_pointer_down_with_shift(world: &mut SelectToolWorld) { world.input_event = Some(pointer_down_event(&world.document, SelectPointerHit::Shape(SelectShapeHit { shape_index: 0, shape_id: world.shape_id }), Vec2::new(5.0, 5.0), true)); }

#[given("the select tool has an active {drag_kind:word} dragging state")]
fn given_active_dragging_state(
    world: &mut SelectToolWorld,
    drag_kind: String,
) -> TestSupportResult<()> {
    let (hit, cursor_world) = resolve_drag_setup(drag_kind.as_str(), world.shape_id)?;
    set_drag_state_from_hit(world, hit, cursor_world)
}

#[given(
    "the select tool event is pointer move with {state:word} state and has_primary_button {has_primary_button:bool}"
)]
fn given_pointer_move(
    world: &mut SelectToolWorld,
    state: String,
    has_primary_button: bool,
) -> TestSupportResult<()> {
    let resolved_state = resolve_state(world, state.as_str())?;
    world.input_event = Some(ToolInputEvent::SelectPointerMove {
        input: Box::new(SelectPointerMoveInput {
            is_dragging: matches!(resolved_state, SelectToolState::Dragging(_)),
            cursor_world: Vec2::new(8.0, 9.0),
            has_primary_button,
        }),
    });
    Ok(())
}

#[given(
    "the select tool event is pointer up at {position:word} with {state:word} state and is_primary_button {is_primary_button:bool}"
)]
fn given_pointer_up(
    world: &mut SelectToolWorld,
    position: String,
    state: String,
    is_primary_button: bool,
) -> TestSupportResult<()> {
    world.input_event = Some(ToolInputEvent::SelectPointerUp {
        input: Box::new(SelectPointerUpInput {
            state: resolve_state(world, state.as_str())?,
            cursor_world: resolve_up_position(position.as_str())?,
            is_primary_button,
        }),
    });
    Ok(())
}

#[when("the select tool transition is evaluated")]
fn when_transition_evaluated(world: &mut SelectToolWorld) -> TestSupportResult<()> {
    let input = world
        .input_event
        .clone()
        .ok_or_else(|| TestSupportError::missing("input_event", "transition evaluation"))?;

    world.transition = Some(Tool::transition(
        &SelectTool,
        world.mode,
        world.edge_mode,
        input,
    ));
    Ok(())
}

#[then("it emits a selection change record")]
fn then_emits_selection_change_record(world: &SelectToolWorld) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands
        .iter()
        .any(|command| matches!(command, ToolCommand::RecordSelectionChange { .. }))
    {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "expected RecordSelectionChange command; got {commands:?}"
    )))
}

#[then("it emits SetSelection for the hit shape")]
fn then_emits_set_selection(world: &SelectToolWorld) -> TestSupportResult<()> {
    assert_emits_command_matching(
        world,
        |command| {
            matches!(
            command,
            ToolCommand::SetSelection(selection)
                if selection.contains(&gauss::model::SelItem::Shape(world.shape_id))
            )
        },
        "SetSelection containing hit shape",
    )
}

#[then("it emits SetSelectToolState Dragging")]
fn then_emits_dragging_state(world: &SelectToolWorld) -> TestSupportResult<()> {
    assert_emits_command_matching(
        world,
        |command| {
            matches!(
                command,
                ToolCommand::SetSelectToolState(SelectToolState::Dragging(_))
            )
        },
        "SetSelectToolState(Dragging)",
    )
}

#[then("it emits SetSelectToolState Idle")]
fn then_emits_idle_state(world: &SelectToolWorld) -> TestSupportResult<()> {
    assert_contains_command(
        world,
        &ToolCommand::SetSelectToolState(SelectToolState::Idle),
    )
}

#[then("it emits PreviewSelectDrag at world position {x:f32} {y:f32}")]
fn then_emits_preview_select_drag(
    world: &SelectToolWorld,
    x: f32,
    y: f32,
) -> TestSupportResult<()> {
    assert_contains_command(
        world,
        &ToolCommand::PreviewSelectDrag {
            cursor_world: Vec2::new(x, y),
        },
    )
}

#[then("it emits RestoreSelectDragPreview")]
fn then_emits_restore_preview(world: &SelectToolWorld) -> TestSupportResult<()> {
    assert_contains_command(world, &ToolCommand::RestoreSelectDragPreview)
}

fn matches_document_command(command: &Command, expected: &str) -> bool {
    match expected {
        "MoveShapes" => matches!(command, Command::MoveShapes { .. }),
        "MoveAnchor" => matches!(command, Command::MoveAnchor { .. }),
        "MoveHandle" => matches!(command, Command::MoveHandle { .. }),
        _ => false,
    }
}

#[then("it emits ApplyDocumentCommand {command:word}")]
fn then_emits_apply_document_command(
    world: &SelectToolWorld,
    command: String,
) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.iter().any(|emitted| {
        matches!(
            emitted,
            ToolCommand::ApplyDocumentCommand(document_command)
                if matches_document_command(document_command.as_ref(), &command)
        )
    }) {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "expected ApplyDocumentCommand({command}); got {commands:?}"
    )))
}

#[then("it emits exactly {count:usize} select tool commands")]
#[rustfmt::skip]
fn then_emits_exact_command_count(world: &SelectToolWorld, count: usize) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.len() == count { return Ok(()); }
    Err(TestSupportError::expectation(format!("expected {count} commands; got {} ({commands:?})", commands.len())))
}

#[then("it emits no select tool commands")]
fn then_emits_no_commands(world: &SelectToolWorld) -> TestSupportResult<()> {
    then_emits_exact_command_count(world, 0)
}

#[path = "select_tool_bdd/scenario_bindings.rs"]
mod scenario_bindings;
