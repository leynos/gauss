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
