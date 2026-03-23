//! BDD step wiring for `SelectTool` scenarios.

use super::SelectToolWorld;
use super::helpers::{
    self, PointerDownEventInput, pointer_down_event, resolve_drag_setup, resolve_state,
    resolve_up_position, set_drag_state_from_hit,
};
use gauss_core::model::{
    SelectPointerHit, SelectPointerMoveInput, SelectPointerUpInput, SelectShapeHit, SelectTool,
    SelectToolState, Tool, ToolInputEvent, Vec2,
};
use rstest_bdd_macros::{given, then, when};
use test_support::TestSupportResult;

#[given("the select tool mode is Manipulate")]
fn given_mode_manipulate(world: &mut SelectToolWorld) {
    world.mode = gauss_core::model::ToolMode::Manipulate;
}

#[given("the select tool mode is Draw")]
fn given_mode_draw(world: &mut SelectToolWorld) {
    world.mode = gauss_core::model::ToolMode::Draw;
}

#[given("the select tool event is pointer down on shape without shift")]
#[rustfmt::skip]
fn given_pointer_down_without_shift(world: &mut SelectToolWorld) { world.input_event = Some(pointer_down_event(&world.document, PointerDownEventInput { hit: SelectPointerHit::Shape(SelectShapeHit { shape_index: 0, shape_id: world.shape_id }), cursor_world: Vec2::new(5.0, 5.0), is_shift_held: false, previous_selection: gauss_core::model::Selection::empty() })); }

#[given("the select tool event is pointer down on shape with shift")]
#[rustfmt::skip]
fn given_pointer_down_with_shift(world: &mut SelectToolWorld) { world.input_event = Some(pointer_down_event(&world.document, PointerDownEventInput { hit: SelectPointerHit::Shape(SelectShapeHit { shape_index: 0, shape_id: world.shape_id }), cursor_world: Vec2::new(5.0, 5.0), is_shift_held: true, previous_selection: gauss_core::model::Selection::empty() })); }

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
    let input = world.input_event.clone().ok_or_else(|| {
        test_support::TestSupportError::missing("input_event", "transition evaluation")
    })?;

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
    helpers::then_emits_selection_change_record(world)
}

#[then("it emits SetSelection for the hit shape")]
fn then_emits_set_selection(world: &SelectToolWorld) -> TestSupportResult<()> {
    helpers::then_emits_set_selection(world)
}

#[then("it emits SetSelectToolState Dragging")]
fn then_emits_dragging_state(world: &SelectToolWorld) -> TestSupportResult<()> {
    helpers::then_emits_dragging_state(world)
}

#[then("it emits SetSelectToolState Idle")]
fn then_emits_idle_state(world: &SelectToolWorld) -> TestSupportResult<()> {
    helpers::then_emits_idle_state(world)
}

#[then("it emits PreviewSelectDrag at world position {x:f32} {y:f32}")]
fn then_emits_preview_select_drag(
    world: &SelectToolWorld,
    x: f32,
    y: f32,
) -> TestSupportResult<()> {
    helpers::then_emits_preview_select_drag(world, x, y)
}

#[then("it emits RestoreSelectDragPreview")]
fn then_emits_restore_preview(world: &SelectToolWorld) -> TestSupportResult<()> {
    helpers::then_emits_restore_preview(world)
}

#[then("it emits ApplyDocumentCommand {command:word}")]
fn then_emits_apply_document_command(
    world: &SelectToolWorld,
    command: String,
) -> TestSupportResult<()> {
    helpers::then_emits_apply_document_command(world, &command)
}

#[then("it emits exactly {count:usize} select tool commands")]
fn then_emits_exact_command_count(world: &SelectToolWorld, count: usize) -> TestSupportResult<()> {
    helpers::then_emits_exact_command_count(world, count)
}

#[then("it emits no select tool commands")]
fn then_emits_no_commands(world: &SelectToolWorld) -> TestSupportResult<()> {
    helpers::then_emits_no_commands(world)
}
