//! Given / When / Then step definitions for undo entry count scenarios.

use gauss::model::history::HistoryError;
use gauss::model::{
    Anchor, Command, DeletedShape, DocumentUndoHistory, HandleKind, HandleMovement, PaintStyle,
    ReorderOp, Rgba, SegmentChange, SegmentKind, ShapeInsertion, ShapeMovement, ShapeReplacement,
    StyleChange, Vec2,
};
use rstest_bdd_macros::{given, then, when};
use test_support::shapes::{open_triangle, sample_shape, shape_id, shape_with_handles};
use test_support::{TestSupportError, TestSupportResult};

use super::{
    EntryCountWorld, apply_and_record, assert_history_length, assert_last_grouping_error,
    get_first_shape, get_first_shape_id,
};

/// Helper to construct and apply a `MoveShapes` command with the given delta.
fn apply_move_shapes_command(
    world: &mut EntryCountWorld,
    delta: Vec2,
    context: &str,
) -> TestSupportResult<()> {
    let shape_id = get_first_shape_id(world, context)?;
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement { shape_id, delta }],
    };
    apply_and_record(world, cmd)
}

fn resolve_grouping_error(expected: &str) -> Option<HistoryError> {
    match expected {
        "group-already-active" => Some(HistoryError::GroupAlreadyActive),
        "no-active-group" => Some(HistoryError::NoActiveGroup),
        "undo-while-group-active" => Some(HistoryError::UndoWhileGroupActive),
        "redo-while-group-active" => Some(HistoryError::RedoWhileGroupActive),
        _ => None,
    }
}

// === Given steps ===

#[given("an empty history and a document with one shape")]
pub(crate) fn given_one_shape(world: &mut EntryCountWorld) {
    world.document = gauss::model::Document::new();
    world.document.append_shape(sample_shape(shape_id(1), 0));
    world.history = DocumentUndoHistory::new();
    world.last_grouping_error = None;
}

#[given("an empty history and an empty document")]
pub(crate) fn given_empty_doc(world: &mut EntryCountWorld) {
    world.document = gauss::model::Document::new();
    world.history = DocumentUndoHistory::new();
    world.last_grouping_error = None;
}

#[given("an empty history and a document with one cubic shape")]
pub(crate) fn given_one_cubic_shape(world: &mut EntryCountWorld) {
    world.document = gauss::model::Document::new();
    world.document.append_shape(shape_with_handles(shape_id(1)));
    world.history = DocumentUndoHistory::new();
    world.last_grouping_error = None;
}

#[given("an empty history and a document with an open triangle")]
pub(crate) fn given_open_triangle(world: &mut EntryCountWorld) {
    world.document = gauss::model::Document::new();
    world.document.append_shape(open_triangle(shape_id(1), 0));
    world.history = DocumentUndoHistory::new();
    world.last_grouping_error = None;
}

#[given("an empty history and a document with two shapes")]
pub(crate) fn given_two_shapes(world: &mut EntryCountWorld) {
    world.document = gauss::model::Document::new();
    world.document.append_shape(sample_shape(shape_id(1), 0));
    world.document.append_shape(sample_shape(shape_id(2), 1));
    world.history = DocumentUndoHistory::new();
    world.last_grouping_error = None;
}

// === When steps ===

#[when("I apply a MoveShapes command")]
pub(crate) fn when_move_shapes(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    apply_move_shapes_command(world, Vec2::new(1.0, 0.0), "MoveShapes")
}

#[when("I begin a command group")]
pub(crate) fn when_begin_command_group(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    world.last_grouping_error = None;
    world.history.begin_group().map_err(|error| {
        test_support::TestSupportError::expectation(format!("begin group failed: {error}"))
    })
}

#[when("I begin another command group")]
pub(crate) fn when_begin_another_command_group(world: &mut EntryCountWorld) {
    world.last_grouping_error = world.history.begin_group().err();
}

#[when("I end the active command group")]
pub(crate) fn when_end_active_command_group(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    world.last_grouping_error = None;
    world.history.end_group().map_err(|error| {
        test_support::TestSupportError::expectation(format!("end group failed: {error}"))
    })
}

#[when("I end a command group without beginning one")]
pub(crate) fn when_end_group_without_begin(world: &mut EntryCountWorld) {
    world.last_grouping_error = world.history.end_group().err();
}

#[when("I undo once")]
pub(crate) fn when_undo_once(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    world.last_grouping_error = None;
    world.history.undo(&mut world.document).map_err(|error| {
        TestSupportError::expectation(format!("undo failed: {error}"))
    })
}

#[when("I attempt to undo while a command group is active")]
pub(crate) fn when_undo_while_group_active(world: &mut EntryCountWorld) {
    world.last_grouping_error = world.history.undo(&mut world.document).err();
}

#[when("I attempt to redo while a command group is active")]
pub(crate) fn when_redo_while_group_active(world: &mut EntryCountWorld) {
    world.last_grouping_error = world.history.redo(&mut world.document).err();
}

#[when("I apply a MoveAnchor command")]
pub(crate) fn when_move_anchor(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "MoveAnchor")?;
    let original = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| test_support::TestSupportError::missing("anchor", "MoveAnchor"))?
        .clone();
    let cmd = Command::MoveAnchor {
        movement: gauss::model::AnchorMovement {
            shape_id: shape.id,
            anchor_index: 0,
            original,
            delta: Vec2::new(1.0, 0.0),
        },
    };
    apply_and_record(world, cmd)
}

#[expect(
    clippy::float_arithmetic,
    reason = "handle position arithmetic requires floating-point operations"
)]
#[when("I apply a MoveHandle command")]
pub(crate) fn when_move_handle(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "MoveHandle")?;
    let anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| test_support::TestSupportError::missing("anchor", "MoveHandle"))?;
    let handle_out = anchor
        .handle_out
        .ok_or_else(|| test_support::TestSupportError::missing("handle_out", "MoveHandle"))?;
    let new_pos = Vec2::new(handle_out.x + 1.0, handle_out.y);
    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: shape.id,
            anchor_index: 0,
            kind: HandleKind::Out,
            from: Some(handle_out),
            to: Some(new_pos),
        },
    };
    apply_and_record(world, cmd)
}

#[when("I apply an InsertShape command")]
pub(crate) fn when_insert_shape(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion {
            index: 0,
            shape: sample_shape(shape_id(10), 0),
        },
    };
    apply_and_record(world, cmd)
}

#[when("I apply a DeleteShapes command")]
pub(crate) fn when_delete_shapes(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "DeleteShapes")?.clone();
    let index = world
        .document
        .find_index(shape.id)
        .ok_or_else(|| test_support::TestSupportError::missing("index", "DeleteShapes"))?;
    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape { index, shape }],
    };
    apply_and_record(world, cmd)
}

#[when("I apply a ClosePath command")]
pub(crate) fn when_close_path(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let old_shape = get_first_shape(world, "ClosePath")?.clone();
    let mut new_shape = old_shape.clone();
    // Close the path using the closing_segment field (no extra segment pushed).
    new_shape.path.closed = true;
    new_shape.path.closing_segment = SegmentKind::Line;
    let cmd = Command::ClosePath {
        replacement: ShapeReplacement {
            shape_index: 0,
            old_shape,
            new_shape,
        },
    };
    apply_and_record(world, cmd)
}

#[when("I apply an InsertAnchor command")]
pub(crate) fn when_insert_anchor(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let old_shape = get_first_shape(world, "InsertAnchor")?.clone();
    let mut new_shape = old_shape.clone();
    // Add a third anchor with a connecting segment.
    new_shape
        .path
        .anchors
        .push(Anchor::new(Vec2::new(50.0, 60.0)));
    new_shape.path.segments.push(SegmentKind::Line);
    let cmd = Command::InsertAnchor {
        replacement: ShapeReplacement {
            shape_index: 0,
            old_shape,
            new_shape,
        },
    };
    apply_and_record(world, cmd)
}

#[expect(
    clippy::float_arithmetic,
    reason = "cubic handle interpolation requires floating-point operations"
)]
#[when("I apply a SetSegmentKind command")]
pub(crate) fn when_set_segment_kind(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "SetSegmentKind")?;
    let start_anchor =
        shape.path.anchors.first().ok_or_else(|| {
            test_support::TestSupportError::missing("start anchor", "SetSegmentKind")
        })?;
    let end_anchor =
        shape.path.anchors.get(1).ok_or_else(|| {
            test_support::TestSupportError::missing("end anchor", "SetSegmentKind")
        })?;
    // Synthesize cubic handles at 1/3 and 2/3 along the segment.
    let dx = end_anchor.pos.x - start_anchor.pos.x;
    let dy = end_anchor.pos.y - start_anchor.pos.y;
    let new_start_out = Vec2::new(start_anchor.pos.x + dx / 3.0, start_anchor.pos.y + dy / 3.0);
    let new_end_in = Vec2::new(end_anchor.pos.x - dx / 3.0, end_anchor.pos.y - dy / 3.0);
    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: shape.id,
            segment_index: 0,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: start_anchor.handle_out,
            new_start_handle_out: Some(new_start_out),
            old_end_handle_in: end_anchor.handle_in,
            new_end_handle_in: Some(new_end_in),
        }],
    };
    apply_and_record(world, cmd)
}

#[when("I apply a Reorder command")]
pub(crate) fn when_reorder(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let id = get_first_shape_id(world, "Reorder")?;
    let cmd = Command::Reorder {
        operations: vec![ReorderOp {
            shape_id: id,
            from_index: 0,
            to_index: 1,
        }],
    };
    apply_and_record(world, cmd)
}

#[when("I apply a SetStyle command")]
pub(crate) fn when_set_style(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "SetStyle")?;
    let cmd = Command::SetStyle {
        changes: vec![StyleChange {
            shape_id: shape.id,
            from: shape.style.clone(),
            to: PaintStyle::new(Some(Rgba::new(0, 255, 0, 255)), 3.0, None),
        }],
    };
    apply_and_record(world, cmd)
}

#[when("I apply another MoveShapes command")]
pub(crate) fn when_another_move_shapes(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    apply_move_shapes_command(world, Vec2::new(0.0, 1.0), "another MoveShapes")
}

// === Then steps ===

#[then("the history length should be {expected:usize}")]
pub(crate) fn then_history_length_is(
    world: &EntryCountWorld,
    expected: usize,
) -> TestSupportResult<()> {
    assert_history_length(world, expected)
}

#[then("the grouping error should be {expected}")]
pub(crate) fn then_grouping_error_is(
    world: &EntryCountWorld,
    expected: String,
) -> TestSupportResult<()> {
    let expected_error = resolve_grouping_error(&expected).ok_or_else(|| {
        TestSupportError::expectation(format!("unknown grouping error token '{expected}'"))
    })?;
    assert_last_grouping_error(world, &expected_error)
}

#[cfg(test)]
mod tests {
    //! Regression coverage for `MoveHandle` step behaviour.

    use gauss::model::{Document, DocumentUndoHistory};

    use super::*;

    fn new_world() -> EntryCountWorld {
        EntryCountWorld {
            document: Document::new(),
            history: DocumentUndoHistory::new(),
            last_grouping_error: None,
        }
    }

    #[test]
    fn move_handle_requires_present_out_handle() {
        let mut world = new_world();
        given_one_shape(&mut world);

        let error = when_move_handle(&mut world)
            .expect_err("MoveHandle should fail when the selected anchor has no out handle");

        assert_eq!(error.to_string(), "missing handle_out: MoveHandle");
        assert_eq!(world.history.len(), 0);
    }

    #[test]
    fn move_handle_records_single_entry_when_handle_exists() -> TestSupportResult<()> {
        let mut world = new_world();
        given_one_cubic_shape(&mut world);

        when_move_handle(&mut world)?;
        assert_history_length(&world, 1)
    }

    #[test]
    fn end_group_without_begin_captures_grouping_error() {
        let mut world = new_world();
        given_one_shape(&mut world);

        when_end_group_without_begin(&mut world);

        assert_last_grouping_error(&world, &HistoryError::NoActiveGroup)
            .expect("expected deterministic grouping error");
        assert_eq!(world.history.len(), 0);
    }

    #[test]
    fn nested_begin_group_captures_grouping_error() -> TestSupportResult<()> {
        let mut world = new_world();
        given_one_shape(&mut world);
        when_begin_command_group(&mut world)?;

        when_begin_another_command_group(&mut world);

        assert_last_grouping_error(&world, &HistoryError::GroupAlreadyActive)?;
        if !world.history.is_empty() {
            return Err(test_support::TestSupportError::expectation(
                "history should remain unchanged on nested begin",
            ));
        }
        when_end_active_command_group(&mut world)
    }
}
