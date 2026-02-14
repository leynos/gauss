//! Behaviour tests for the single-entry-per-gesture undo invariant.
//!
//! Each command type must produce exactly one undo entry when applied
//! through [`DocumentUndoHistory`], ensuring that a single user gesture
//! always corresponds to a single undoable step.
#![expect(
    clippy::float_arithmetic,
    reason = "geometry calculations require arithmetic in test fixtures"
)]

use gauss::model::{
    Anchor, Command, DeletedShape, Document, DocumentUndoHistory, HandleKind, HandleMovement,
    PaintStyle, PathGeom, ReorderOp, Rgba, SegmentChange, SegmentKind, ShapeInsertion,
    ShapeMovement, ShapeReplacement, StyleChange, Vec2,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::shapes::{sample_shape, shape_id, shape_with_handles};
use test_support::{TestSupportError, TestSupportResult};

/// World state for undo entry count BDD tests.
struct EntryCountWorld {
    document: Document,
    history: DocumentUndoHistory,
}

#[fixture]
fn world() -> EntryCountWorld {
    EntryCountWorld {
        document: Document::new(),
        history: DocumentUndoHistory::new(),
    }
}

// === Helper functions ===

fn assert_history_length(world: &EntryCountWorld, expected: usize) -> TestSupportResult<()> {
    if world.history.len() == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected history length {expected}, got {}",
            world.history.len()
        )))
    }
}

fn get_first_shape<'a>(
    world: &'a EntryCountWorld,
    context: &str,
) -> TestSupportResult<&'a gauss::model::Shape> {
    world
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", context))
}

fn get_first_shape_id(
    world: &EntryCountWorld,
    context: &str,
) -> TestSupportResult<gauss::model::ShapeId> {
    world
        .document
        .shape_id_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", context))
}

// === Given steps ===

#[given("an empty history and a document with one shape")]
fn given_one_shape(world: &mut EntryCountWorld) {
    world.document = Document::new();
    world.document.append_shape(sample_shape(shape_id(1), 0));
    world.history = DocumentUndoHistory::new();
}

#[given("an empty history and an empty document")]
fn given_empty_doc(world: &mut EntryCountWorld) {
    world.document = Document::new();
    world.history = DocumentUndoHistory::new();
}

#[given("an empty history and a document with one cubic shape")]
fn given_one_cubic_shape(world: &mut EntryCountWorld) {
    world.document = Document::new();
    world.document.append_shape(shape_with_handles(shape_id(1)));
    world.history = DocumentUndoHistory::new();
}

#[given("an empty history and a document with an open triangle")]
fn given_open_triangle(world: &mut EntryCountWorld) {
    world.document = Document::new();
    let mut path = PathGeom::new();
    path.anchors.push(Anchor::new(Vec2::new(0.0, 0.0)));
    path.anchors.push(Anchor::new(Vec2::new(10.0, 0.0)));
    path.anchors.push(Anchor::new(Vec2::new(5.0, 10.0)));
    path.segments.push(SegmentKind::Line);
    path.segments.push(SegmentKind::Line);

    let shape = gauss::model::Shape {
        id: shape_id(1),
        z: 0,
        style: PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), 2.0, None),
        path,
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    };
    world.document.append_shape(shape);
    world.history = DocumentUndoHistory::new();
}

#[given("an empty history and a document with two shapes")]
fn given_two_shapes(world: &mut EntryCountWorld) {
    world.document = Document::new();
    world.document.append_shape(sample_shape(shape_id(1), 0));
    world.document.append_shape(sample_shape(shape_id(2), 1));
    world.history = DocumentUndoHistory::new();
}

// === When steps ===

/// Apply a command to the document and record it in the undo history.
fn apply_and_record(world: &mut EntryCountWorld, cmd: Command) -> TestSupportResult<()> {
    let inverse = cmd
        .apply(&mut world.document)
        .map_err(|e| TestSupportError::expectation(format!("apply failed: {e}")))?;
    world.history.record(cmd, inverse);
    Ok(())
}

#[when("I apply a MoveShapes command")]
fn when_move_shapes(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape_id = get_first_shape_id(world, "MoveShapes")?;
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id,
            delta: Vec2::new(1.0, 0.0),
        }],
    };
    apply_and_record(world, cmd)
}

#[when("I apply a MoveAnchor command")]
fn when_move_anchor(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "MoveAnchor")?;
    let original = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor", "MoveAnchor"))?
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

#[when("I apply a MoveHandle command")]
fn when_move_handle(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "MoveHandle")?;
    let anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor", "MoveHandle"))?;
    let handle_out = anchor.handle_out;
    let new_pos = handle_out.map(|h| Vec2::new(h.x + 1.0, h.y));
    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: shape.id,
            anchor_index: 0,
            kind: HandleKind::Out,
            from: handle_out,
            to: new_pos,
        },
    };
    apply_and_record(world, cmd)
}

#[when("I apply an InsertShape command")]
fn when_insert_shape(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion {
            index: 0,
            shape: sample_shape(shape_id(10), 0),
        },
    };
    apply_and_record(world, cmd)
}

#[when("I apply a DeleteShapes command")]
fn when_delete_shapes(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "DeleteShapes")?.clone();
    let index = world
        .document
        .find_index(shape.id)
        .ok_or_else(|| TestSupportError::missing("index", "DeleteShapes"))?;
    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape { index, shape }],
    };
    apply_and_record(world, cmd)
}

#[when("I apply a ClosePath command")]
fn when_close_path(world: &mut EntryCountWorld) -> TestSupportResult<()> {
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
fn when_insert_anchor(world: &mut EntryCountWorld) -> TestSupportResult<()> {
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

#[when("I apply a SetSegmentKind command")]
fn when_set_segment_kind(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape = get_first_shape(world, "SetSegmentKind")?;
    let start_anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("start anchor", "SetSegmentKind"))?;
    let end_anchor = shape
        .path
        .anchors
        .get(1)
        .ok_or_else(|| TestSupportError::missing("end anchor", "SetSegmentKind"))?;
    // Synthesise cubic handles at 1/3 and 2/3 along the segment.
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
fn when_reorder(world: &mut EntryCountWorld) -> TestSupportResult<()> {
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
fn when_set_style(world: &mut EntryCountWorld) -> TestSupportResult<()> {
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
fn when_another_move_shapes(world: &mut EntryCountWorld) -> TestSupportResult<()> {
    let shape_id = get_first_shape_id(world, "another MoveShapes")?;
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id,
            delta: Vec2::new(0.0, 1.0),
        }],
    };
    apply_and_record(world, cmd)
}

// === Then steps ===

#[then("the history length should be 1")]
fn then_history_length_is_one(world: &EntryCountWorld) -> TestSupportResult<()> {
    assert_history_length(world, 1)
}

#[then("the history length should be 3")]
fn then_history_length_is_three(world: &EntryCountWorld) -> TestSupportResult<()> {
    assert_history_length(world, 3)
}

// === Scenario bindings ===

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "MoveShapes command produces single undo entry"
)]
fn move_shapes_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "MoveAnchor command produces single undo entry"
)]
fn move_anchor_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "MoveHandle command produces single undo entry"
)]
fn move_handle_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "InsertShape command produces single undo entry"
)]
fn insert_shape_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "DeleteShapes command produces single undo entry"
)]
fn delete_shapes_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "ClosePath command produces single undo entry"
)]
fn close_path_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "InsertAnchor command produces single undo entry"
)]
fn insert_anchor_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "SetSegmentKind command produces single undo entry"
)]
fn set_segment_kind_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Reorder command produces single undo entry"
)]
fn reorder_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "SetStyle command produces single undo entry"
)]
fn set_style_command_produces_single_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Multiple sequential commands produce matching undo count"
)]
fn multiple_sequential_commands_produce_matching_undo_count(world: EntryCountWorld) {
    let _ = world;
}
