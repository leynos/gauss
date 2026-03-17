//! Behaviour tests for `PenTool` canvas-click transitions.
//!
//! These scenarios lock draw-mode command-emission parity while Pen behaviour
//! is routed through the Tool trait boundary.

use gauss::model::{
    Anchor, EdgeMode, PathGeom, PenTool, PenToolActiveShape, PenToolClickInput, SegmentKind, Shape,
    ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode, ToolTransition, Vec2,
};
use gauss_core::test_helpers::sample_style;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct PenToolWorld {
    mode: ToolMode,
    edge_mode: EdgeMode,
    active_path: Option<ShapeId>,
    active_shape: Option<PenToolActiveShape>,
    cursor_world: Option<Vec2>,
    transition: Option<ToolTransition>,
}

#[fixture]
fn world() -> PenToolWorld {
    PenToolWorld {
        mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        active_path: None,
        active_shape: None,
        cursor_world: None,
        transition: None,
    }
}

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn open_shape(id: ShapeId, anchors: Vec<Vec2>, edge_mode: EdgeMode) -> Shape {
    let segment_kind = match edge_mode {
        EdgeMode::Line => SegmentKind::Line,
        EdgeMode::BezierAuto => SegmentKind::Cubic,
    };
    let anchor_count = anchors.len();

    Shape {
        id,
        z: 0,
        style: sample_style(),
        path: PathGeom {
            anchors: anchors.into_iter().map(Anchor::new).collect(),
            segments: vec![segment_kind; anchor_count.saturating_sub(1)],
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

fn set_active_open_path(world: &mut PenToolWorld, id: ShapeId, index: usize, anchors: Vec<Vec2>) {
    world.active_path = Some(id);
    world.active_shape = Some(PenToolActiveShape {
        index,
        shape: open_shape(id, anchors, world.edge_mode),
    });
}

fn assert_command_emitted(
    world: &PenToolWorld,
    predicate: impl FnMut(&ToolCommand) -> bool,
    error_message: &str,
) -> TestSupportResult<()> {
    if transition(world)?.commands.iter().any(predicate) {
        return Ok(());
    }

    Err(TestSupportError::expectation(error_message))
}

#[given("the pen tool mode is Draw")]
fn given_tool_mode_draw(world: &mut PenToolWorld) {
    world.mode = ToolMode::Draw;
}

#[given("the pen tool mode is Manipulate")]
fn given_tool_mode_manipulate(world: &mut PenToolWorld) {
    world.mode = ToolMode::Manipulate;
}

#[given("the pen edge mode is Line")]
fn given_edge_mode_line(world: &mut PenToolWorld) {
    world.edge_mode = EdgeMode::Line;
}

#[given("the pen edge mode is BezierAuto")]
fn given_edge_mode_bezier(world: &mut PenToolWorld) {
    world.edge_mode = EdgeMode::BezierAuto;
}

#[given("no pen active path exists")]
fn given_no_active_path(world: &mut PenToolWorld) {
    world.active_path = None;
    world.active_shape = None;
}

#[given("a stale pen active path exists")]
fn given_stale_active_path(world: &mut PenToolWorld) {
    world.active_path = Some(shape_id(700));
    world.active_shape = None;
}

#[given("a pen active open path with 2 anchors exists")]
fn given_active_open_path_two_anchors(world: &mut PenToolWorld) {
    set_active_open_path(
        world,
        shape_id(701),
        5,
        vec![Vec2::new(0.0, 0.0), Vec2::new(20.0, 0.0)],
    );
}

#[given("a pen active open path with 3 anchors exists")]
fn given_active_open_path_three_anchors(world: &mut PenToolWorld) {
    set_active_open_path(
        world,
        shape_id(702),
        6,
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            Vec2::new(20.0, 25.0),
        ],
    );
}

#[given("the pen click is at ({x:f32}, {y:f32})")]
fn given_click_at(world: &mut PenToolWorld, x: f32, y: f32) {
    world.cursor_world = Some(Vec2::new(x, y));
}

#[when("the pen transition is evaluated")]
fn when_pen_transition_is_evaluated(world: &mut PenToolWorld) -> TestSupportResult<()> {
    let cursor_world = world
        .cursor_world
        .ok_or_else(|| TestSupportError::missing("cursor_world", "pen transition"))?;

    let input = PenToolClickInput {
        cursor_world,
        zoom: 1.0,
        current_style: sample_style(),
        active_path: world.active_path,
        active_shape: world.active_shape.clone(),
        next_shape_id: shape_id(999),
        document_len: 3,
        snap_radius_px: PenToolClickInput::DEFAULT_SNAP_RADIUS_PX,
    };

    world.transition = Some(Tool::transition(
        &PenTool,
        world.mode,
        world.edge_mode,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input),
        },
    ));

    Ok(())
}

fn transition(world: &PenToolWorld) -> TestSupportResult<&ToolTransition> {
    world
        .transition
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("transition", "assertion"))
}

fn assert_command_count(world: &PenToolWorld, expected_count: usize) -> TestSupportResult<()> {
    let commands = &transition(world)?.commands;
    if commands.len() == expected_count {
        return Ok(());
    }

    Err(TestSupportError::expectation(format!(
        "Expected {expected_count} commands; got {} ({commands:?})",
        commands.len()
    )))
}

#[then("the pen transition should emit InsertShape")]
fn then_emit_insert_shape(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_emitted(
        world,
        |command| {
            matches!(command, ToolCommand::ApplyDocumentCommand(document_command) if matches!(
                document_command.as_ref(),
                gauss::model::Command::InsertShape { .. }
            ))
        },
        "expected InsertShape command to be emitted",
    )
}

#[then("the pen transition should emit InsertAnchor")]
fn then_emit_insert_anchor(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_emitted(
        world,
        |command| {
            matches!(command, ToolCommand::ApplyDocumentCommand(document_command) if matches!(
                document_command.as_ref(),
                gauss::model::Command::InsertAnchor { .. }
            ))
        },
        "expected InsertAnchor command to be emitted",
    )
}

#[then("the pen transition should emit ClosePath")]
fn then_emit_close_path(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_emitted(
        world,
        |command| {
            matches!(command, ToolCommand::ApplyDocumentCommand(document_command) if matches!(
                document_command.as_ref(),
                gauss::model::Command::ClosePath { .. }
            ))
        },
        "expected ClosePath command to be emitted",
    )
}

#[then("the pen transition should emit SetToolMode Manipulate")]
fn then_emit_set_tool_mode_manipulate(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_emitted(
        world,
        |command| matches!(command, ToolCommand::SetToolMode(ToolMode::Manipulate)),
        "expected SetToolMode(Manipulate) to be emitted",
    )
}

#[then("the pen transition should emit SetActivePath None")]
fn then_emit_set_active_path_none(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_emitted(
        world,
        |command| matches!(command, ToolCommand::SetActivePath(None)),
        "expected SetActivePath(None) to be emitted",
    )
}

#[then("the pen transition should emit SetActivePath Some")]
fn then_emit_set_active_path_some(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_emitted(
        world,
        |command| matches!(command, ToolCommand::SetActivePath(Some(_))),
        "expected SetActivePath(Some(_)) to be emitted",
    )
}

#[then("the pen transition should emit no commands")]
fn then_emit_no_commands(world: &PenToolWorld) -> TestSupportResult<()> {
    assert_command_count(world, 0)
}

#[then("the pen transition should emit exactly {count:usize} commands")]
fn then_emit_exactly_n_commands(world: &PenToolWorld, count: usize) -> TestSupportResult<()> {
    assert_command_count(world, count)
}

#[then("the pen transition should emit exactly {count:usize} command")]
fn then_emit_exactly_n_command(world: &PenToolWorld, count: usize) -> TestSupportResult<()> {
    assert_command_count(world, count)
}

#[scenario(
    path = "tests/features/pen_tool.feature",
    name = "Click with no active path starts a new open shape"
)]
fn click_no_active_path_starts_new_open_shape(world: PenToolWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/pen_tool.feature",
    name = "Click with active open path appends anchor"
)]
fn click_with_active_path_appends_anchor(world: PenToolWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/pen_tool.feature",
    name = "Click near first anchor closes path and exits draw mode"
)]
fn click_near_first_anchor_closes_path(world: PenToolWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/pen_tool.feature",
    name = "Stale active path is cleared then new shape starts"
)]
fn stale_active_path_recovers_and_starts_shape(world: PenToolWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/pen_tool.feature",
    name = "Click while in manipulate mode is ignored"
)]
fn click_in_manipulate_mode_is_ignored(world: PenToolWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/pen_tool.feature",
    name = "Close attempt with two anchors appends instead of closing"
)]
fn close_attempt_with_two_anchors_appends(world: PenToolWorld) {
    let _ = world;
}
