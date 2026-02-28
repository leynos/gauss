//! Behaviour tests for the single-entry-per-gesture undo invariant.
//!
//! Each command type must produce exactly one undo entry when applied
//! through [`DocumentUndoHistory`], ensuring that a single user gesture
//! always corresponds to a single undoable step.

mod steps;

use gauss::model::history::HistoryError;
use gauss::model::{Command, Document, DocumentUndoHistory};
use rstest::fixture;
use rstest_bdd_macros::scenario;
use test_support::{TestSupportError, TestSupportResult};

/// World state for undo entry count BDD tests.
pub(crate) struct EntryCountWorld {
    pub(crate) document: Document,
    pub(crate) history: DocumentUndoHistory,
    pub(crate) last_grouping_error: Option<HistoryError>,
}

#[fixture]
fn world() -> EntryCountWorld {
    EntryCountWorld {
        document: Document::new(),
        history: DocumentUndoHistory::new(),
        last_grouping_error: None,
    }
}

// === Helper functions ===

pub(crate) fn assert_history_length(
    world: &EntryCountWorld,
    expected: usize,
) -> TestSupportResult<()> {
    if world.history.len() == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected history length {expected}, got {}",
            world.history.len()
        )))
    }
}

pub(crate) fn get_first_shape<'a>(
    world: &'a EntryCountWorld,
    context: &str,
) -> TestSupportResult<&'a gauss::model::Shape> {
    world
        .document
        .shape_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", context))
}

pub(crate) fn get_first_shape_id(
    world: &EntryCountWorld,
    context: &str,
) -> TestSupportResult<gauss::model::ShapeId> {
    world
        .document
        .shape_id_at(0)
        .ok_or_else(|| TestSupportError::missing("shape", context))
}

pub(crate) fn assert_last_grouping_error(
    world: &EntryCountWorld,
    expected: &HistoryError,
) -> TestSupportResult<()> {
    let actual = world.last_grouping_error.as_ref().ok_or_else(|| {
        TestSupportError::expectation("expected a grouping error, but none was captured")
    })?;
    if actual == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected grouping error '{expected:?}', got '{actual:?}'"
        )))
    }
}

/// Apply a command to the document and record it in the undo history.
pub(crate) fn apply_and_record(world: &mut EntryCountWorld, cmd: Command) -> TestSupportResult<()> {
    let inverse = cmd
        .apply(&mut world.document)
        .map_err(|e| TestSupportError::expectation(format!("apply failed: {e}")))?;
    world.history.record(cmd, inverse);
    Ok(())
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

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Grouped commands collapse to one undo entry"
)]
fn grouped_commands_collapse_to_one_undo_entry(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Ending a group without begin reports an error and keeps history unchanged"
)]
fn ending_group_without_begin_reports_an_error_and_keeps_history_unchanged(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Nested group begin reports an error and keeps history unchanged"
)]
fn nested_group_begin_reports_an_error_and_keeps_history_unchanged(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Undo while group is active reports an error and keeps history unchanged"
)]
fn undo_while_group_is_active_reports_an_error_and_keeps_history_unchanged(world: EntryCountWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/undo_entry_count.feature",
    name = "Redo while group is active reports an error and keeps history unchanged"
)]
fn redo_while_group_is_active_reports_an_error_and_keeps_history_unchanged(world: EntryCountWorld) {
    let _ = world;
}
