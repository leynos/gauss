//! Helper functions for `SelectTool` BDD steps and assertions.

use super::SelectToolWorld;
use gauss::model::{
    Command, Document, SelectAnchorHit, SelectDragDocumentSnapshot, SelectHandleHit,
    SelectHandleHitKind, SelectPointerDownInput, SelectPointerHit, SelectShapeHit, SelectTool,
    SelectToolState, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolTransition, Vec2,
};
use test_support::{TestSupportError, TestSupportResult};

pub(crate) struct PointerDownEventInput {
    pub(crate) hit: SelectPointerHit,
    pub(crate) cursor_world: Vec2,
    pub(crate) is_shift_held: bool,
    pub(crate) previous_selection: gauss::model::Selection,
}

pub(crate) fn pointer_down_event(
    document: &Document,
    input: PointerDownEventInput,
) -> ToolInputEvent {
    ToolInputEvent::SelectPointerDown {
        input: Box::new(SelectPointerDownInput {
            drag_snapshot: SelectDragDocumentSnapshot::from_document(document),
            previous_selection: input.previous_selection,
            hit: input.hit,
            cursor_world: input.cursor_world,
            is_shift_held: input.is_shift_held,
        }),
    }
}

pub(crate) fn extract_drag_state(transition: &ToolTransition) -> Option<SelectToolState> {
    transition
        .commands
        .iter()
        .find_map(|command| match command {
            ToolCommand::SetSelectToolState(state) => Some(state.clone()),
            _ => None,
        })
}

pub(crate) fn transition(world: &SelectToolWorld) -> TestSupportResult<&ToolTransition> {
    world
        .transition
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("transition", "assertion"))
}

pub(crate) fn assert_contains_command(
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

pub(crate) fn assert_emits_command_matching<M>(
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

pub(crate) fn set_drag_state_from_hit(
    world: &mut SelectToolWorld,
    hit: SelectPointerHit,
    cursor_world: Vec2,
) -> TestSupportResult<()> {
    let previous_selection = match hit {
        SelectPointerHit::Shape(SelectShapeHit { shape_id, .. }) => gauss::model::Selection {
            items: vec![gauss::model::SelItem::Shape(shape_id)],
        },
        _ => gauss::model::Selection::empty(),
    };
    let down_event = pointer_down_event(
        &world.document,
        PointerDownEventInput {
            hit,
            cursor_world,
            is_shift_held: false,
            previous_selection,
        },
    );
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

pub(crate) fn resolve_drag_setup(
    drag_kind: &str,
    shape_id: ShapeId,
) -> TestSupportResult<(SelectPointerHit, Vec2)> {
    match drag_kind {
        "shape" => Ok((
            SelectPointerHit::Shape(SelectShapeHit {
                shape_index: 0,
                shape_id,
            }),
            Vec2::new(5.0, 5.0),
        )),
        "anchor" => Ok((
            SelectPointerHit::Anchor(SelectAnchorHit {
                shape_index: 0,
                shape_id,
                anchor_index: 0,
            }),
            Vec2::new(0.0, 0.0),
        )),
        "handle" => Ok((
            SelectPointerHit::Handle(SelectHandleHit {
                shape_index: 0,
                shape_id,
                anchor_index: 0,
                kind: SelectHandleHitKind::Out,
            }),
            Vec2::new(2.0, 0.0),
        )),
        _ => Err(TestSupportError::expectation(format!(
            "unsupported drag kind '{drag_kind}'"
        ))),
    }
}

pub(crate) fn resolve_state(
    world: &SelectToolWorld,
    state: &str,
) -> TestSupportResult<SelectToolState> {
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

pub(crate) fn resolve_up_position(position: &str) -> TestSupportResult<Vec2> {
    match position {
        "origin" => Ok(Vec2::new(5.0, 5.0)),
        "moved" => Ok(Vec2::new(11.0, 7.0)),
        _ => Err(TestSupportError::expectation(format!(
            "unsupported pointer-up position '{position}'"
        ))),
    }
}

pub(crate) fn then_emits_selection_change_record(world: &SelectToolWorld) -> TestSupportResult<()> {
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

pub(crate) fn then_emits_set_selection(world: &SelectToolWorld) -> TestSupportResult<()> {
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

pub(crate) fn then_emits_dragging_state(world: &SelectToolWorld) -> TestSupportResult<()> {
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

pub(crate) fn then_emits_idle_state(world: &SelectToolWorld) -> TestSupportResult<()> {
    assert_contains_command(
        world,
        &ToolCommand::SetSelectToolState(SelectToolState::Idle),
    )
}

pub(crate) fn then_emits_preview_select_drag(
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

pub(crate) fn then_emits_restore_preview(world: &SelectToolWorld) -> TestSupportResult<()> {
    assert_contains_command(world, &ToolCommand::RestoreSelectDragPreview)
}

pub(crate) fn matches_document_command(command: &Command, expected: &str) -> bool {
    match expected {
        "MoveShapes" => matches!(command, Command::MoveShapes { .. }),
        "MoveAnchor" => matches!(command, Command::MoveAnchor { .. }),
        "MoveHandle" => matches!(command, Command::MoveHandle { .. }),
        _ => false,
    }
}

pub(crate) fn then_emits_apply_document_command(
    world: &SelectToolWorld,
    command: &str,
) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.iter().any(|emitted| {
        matches!(
            emitted,
            ToolCommand::ApplyDocumentCommand(document_command)
                if matches_document_command(document_command.as_ref(), command)
        )
    }) {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "expected ApplyDocumentCommand({command}); got {commands:?}"
    )))
}

pub(crate) fn then_emits_exact_command_count(
    world: &SelectToolWorld,
    count: usize,
) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.len() == count {
        return Ok(());
    }
    Err(TestSupportError::expectation(format!(
        "expected {count} commands; got {} ({commands:?})",
        commands.len()
    )))
}

pub(crate) fn then_emits_no_commands(world: &SelectToolWorld) -> TestSupportResult<()> {
    then_emits_exact_command_count(world, 0)
}
