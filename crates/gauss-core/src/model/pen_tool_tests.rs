//! Unit tests for `PenTool` draw-click transitions.

use super::{
    Anchor, Command, EdgeMode, Paint, PaintStyle, PathGeom, PenTool, PenToolActiveShape,
    PenToolClickInput, SegmentKind, Shape, ShapeId, Tool, ToolCommand, ToolInputEvent, ToolMode,
    Vec2,
};
use rstest::rstest;

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn default_style(fill: Paint) -> PaintStyle {
    PaintStyle {
        stroke: Paint::Solid(super::Rgba::new(16, 32, 64, 255)),
        stroke_width: 2.0,
        fill,
    }
}

fn open_shape(id: ShapeId, anchors: Vec<Vec2>, segment_kind: SegmentKind, fill: Paint) -> Shape {
    let anchor_count = anchors.len();
    Shape {
        id,
        z: 0,
        style: default_style(fill),
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

fn click_input(cursor_world: Vec2) -> PenToolClickInput {
    PenToolClickInput {
        cursor_world,
        zoom: 1.0,
        current_style: default_style(Paint::None),
        active_path: None,
        active_shape: None,
        next_shape_id: shape_id(99),
        document_len: 3,
        snap_radius_px: PenToolClickInput::DEFAULT_SNAP_RADIUS_PX,
    }
}

#[rstest]
fn pen_tool_starts_new_shape_when_no_active_path() {
    let tool = PenTool;
    let input = click_input(Vec2::new(20.0, 30.0));

    let transition = Tool::transition(
        &tool,
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input.clone()),
        },
    );

    let [
        ToolCommand::ApplyDocumentCommand(command),
        ToolCommand::SetActivePath(Some(active_path)),
    ] = transition.commands.as_slice()
    else {
        panic!(
            "expected InsertShape + SetActivePath(Some) commands; got {:?}",
            transition.commands
        );
    };

    match command.as_ref() {
        Command::InsertShape { insertion } => {
            assert_eq!(insertion.index, input.document_len);
            assert_eq!(insertion.shape.id, input.next_shape_id);
            assert_eq!(insertion.shape.path.anchors.len(), 1);
            let Some(first_anchor) = insertion.shape.path.anchors.first() else {
                panic!("expected first anchor in new shape")
            };
            assert_eq!(first_anchor.pos, input.cursor_world);
            assert!(insertion.shape.style.fill.is_none());
        }
        other => panic!("expected InsertShape, got {other:?}"),
    }
    assert_eq!(*active_path, input.next_shape_id);
}

#[rstest]
fn pen_tool_recovers_from_stale_active_path_then_starts_new_shape() {
    let tool = PenTool;
    let mut input = click_input(Vec2::new(8.0, 9.0));
    input.active_path = Some(shape_id(7));

    let transition = Tool::transition(
        &tool,
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input.clone()),
        },
    );

    let [
        ToolCommand::SetActivePath(None),
        ToolCommand::ApplyDocumentCommand(insert_shape_command),
        ToolCommand::SetActivePath(Some(next_active_path)),
    ] = transition.commands.as_slice()
    else {
        panic!(
            "expected stale recovery command sequence; got {:?}",
            transition.commands
        );
    };

    assert!(matches!(
        insert_shape_command.as_ref(),
        Command::InsertShape { .. }
    ));
    assert_eq!(*next_active_path, input.next_shape_id);
}

#[rstest]
fn pen_tool_recovers_when_active_shape_id_does_not_match_active_path() {
    let tool = PenTool;
    let mut input = click_input(Vec2::new(1.0, 1.0));
    input.active_path = Some(shape_id(11));
    input.active_shape = Some(PenToolActiveShape {
        index: 1,
        shape: open_shape(
            shape_id(22),
            vec![Vec2::new(2.0, 2.0), Vec2::new(3.0, 3.0)],
            SegmentKind::Line,
            Paint::None,
        ),
    });

    let transition = Tool::transition(
        &tool,
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input),
        },
    );

    let [
        ToolCommand::SetActivePath(None),
        ToolCommand::ApplyDocumentCommand(insert_shape_command),
        ToolCommand::SetActivePath(Some(_)),
    ] = transition.commands.as_slice()
    else {
        panic!(
            "expected id-mismatch recovery sequence; got {:?}",
            transition.commands
        );
    };

    assert!(matches!(
        insert_shape_command.as_ref(),
        Command::InsertShape { .. }
    ));
}

#[rstest]
fn pen_tool_appends_anchor_when_active_shape_exists_and_close_not_requested() {
    let tool = PenTool;
    let active_shape = open_shape(
        shape_id(41),
        vec![Vec2::new(0.0, 0.0), Vec2::new(20.0, 0.0)],
        SegmentKind::Line,
        Paint::None,
    );

    let mut input = click_input(Vec2::new(60.0, 24.0));
    input.active_path = Some(active_shape.id);
    input.active_shape = Some(PenToolActiveShape {
        index: 5,
        shape: active_shape.clone(),
    });

    let transition = Tool::transition(
        &tool,
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input),
        },
    );

    let [ToolCommand::ApplyDocumentCommand(command)] = transition.commands.as_slice() else {
        panic!(
            "expected one InsertAnchor document command; got {:?}",
            transition.commands
        );
    };

    match command.as_ref() {
        Command::InsertAnchor { replacement } => {
            assert_eq!(replacement.shape_index, 5);
            assert_eq!(replacement.old_shape, active_shape);
            assert_eq!(replacement.new_shape.path.anchors.len(), 3);
            assert_eq!(replacement.new_shape.path.segments.len(), 2);
            let Some(new_anchor) = replacement.new_shape.path.anchors.get(2) else {
                panic!("expected third anchor after append")
            };
            assert_eq!(new_anchor.pos, Vec2::new(60.0, 24.0));
        }
        other => panic!("expected InsertAnchor command, got {other:?}"),
    }
}

#[rstest]
fn pen_tool_closes_path_and_switches_to_manipulate_when_click_is_within_snap_radius() {
    let tool = PenTool;
    let active_shape = open_shape(
        shape_id(55),
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            Vec2::new(20.0, 25.0),
        ],
        SegmentKind::Line,
        Paint::None,
    );

    let mut input = click_input(Vec2::new(4.0, 3.0));
    input.active_path = Some(active_shape.id);
    input.active_shape = Some(PenToolActiveShape {
        index: 2,
        shape: active_shape.clone(),
    });

    let transition = Tool::transition(
        &tool,
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input),
        },
    );

    let [
        ToolCommand::ApplyDocumentCommand(command),
        ToolCommand::SetToolMode(ToolMode::Manipulate),
        ToolCommand::SetActivePath(None),
    ] = transition.commands.as_slice()
    else {
        panic!(
            "expected close + mode-exit command sequence; got {:?}",
            transition.commands
        );
    };

    match command.as_ref() {
        Command::ClosePath { replacement } => {
            assert_eq!(replacement.shape_index, 2);
            assert_eq!(replacement.old_shape, active_shape);
            assert!(replacement.new_shape.path.closed);
            assert_eq!(
                replacement.new_shape.path.closing_segment,
                SegmentKind::Line
            );
            assert!(!replacement.new_shape.style.fill.is_none());
        }
        other => panic!("expected ClosePath command, got {other:?}"),
    }
}

#[rstest]
#[case::too_few_anchors(
    open_shape(
        shape_id(71),
        vec![Vec2::new(0.0, 0.0), Vec2::new(12.0, 2.0)],
        SegmentKind::Line,
        Paint::None,
    ),
    Vec2::new(1.0, 1.0),
)]
#[case::outside_snap_radius(
    open_shape(
        shape_id(72),
        vec![Vec2::new(0.0, 0.0), Vec2::new(20.0, 0.0), Vec2::new(25.0, 20.0)],
        SegmentKind::Line,
        Paint::None,
    ),
    Vec2::new(80.0, 80.0),
)]
fn pen_tool_does_not_close_path_when_close_conditions_are_not_met(
    #[case] shape: Shape,
    #[case] cursor_world: Vec2,
) {
    let tool = PenTool;
    let mut input = click_input(cursor_world);
    input.active_path = Some(shape.id);
    input.active_shape = Some(PenToolActiveShape { index: 4, shape });

    let transition = Tool::transition(
        &tool,
        ToolMode::Draw,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input),
        },
    );

    let [ToolCommand::ApplyDocumentCommand(command)] = transition.commands.as_slice() else {
        panic!(
            "expected single InsertAnchor command; got {:?}",
            transition.commands
        );
    };

    assert!(matches!(command.as_ref(), Command::InsertAnchor { .. }));
}

#[rstest]
fn pen_tool_ignores_canvas_click_when_not_in_draw_mode() {
    let tool = PenTool;
    let input = click_input(Vec2::new(5.0, 6.0));

    let transition = Tool::transition(
        &tool,
        ToolMode::Manipulate,
        EdgeMode::Line,
        ToolInputEvent::PenCanvasClicked {
            input: Box::new(input),
        },
    );

    assert!(transition.commands.is_empty());
}
