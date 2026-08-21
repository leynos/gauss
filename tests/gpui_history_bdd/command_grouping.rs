//! BDD step bindings for document command grouping history.
//!
//! The steps cover successful grouped moves and invalid group transitions,
//! matching the command-grouping feature scenarios. The parent integration
//! binary runs each scenario with `GpuiHarness`, and this module combines
//! shared document/history helpers with the durable shell from the common
//! history BDD support.

use std::cell::RefCell;

use gpui::TestAppContext;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[path = "command_grouping_active.rs"]
mod active_group;

#[derive(Default)]
struct GroupingState {
    shell: Option<DurableShell>,
    shape: Option<ShapeId>,
    history_before: Option<usize>,
    history_state_before: Option<DocumentHistoryState>,
    document_before: Option<Document>,
    anchor_before: Option<Vec2>,
    error: Option<HistoryError>,
    operation: Option<HistoryOperation>,
}

thread_local! {
    static STATE: RefCell<GroupingState> = RefCell::new(GroupingState::default());
}

/// Apply a closure to the command-grouping scenario state.
fn with_state<R>(f: impl FnOnce(&mut GroupingState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

/// Reset all command-grouping state before or after a scenario.
fn reset_state() {
    with_state(|state| *state = GroupingState::default());
}

struct StateCleanup;

impl Drop for StateCleanup {
    /// Clear thread-local state when the scenario guard is dropped.
    fn drop(&mut self) {
        reset_state();
    }
}

/// Reset state and return the scenario cleanup guard.
#[fixture]
fn state_cleanup() -> StateCleanup {
    reset_state();
    StateCleanup
}

/// Retrieve the durable shell stored by the Given step.
fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| missing("Phase 0 shell"))
}

/// Retrieve the history length recorded during scenario setup.
fn history_before() -> Result<usize, TestSupportError> {
    with_state(|state| state.history_before).ok_or_else(|| missing("initial history length"))
}

/// Begin a document command group and surface any history error.
fn begin_group(durable_shell: &DurableShell, cx: &mut TestAppContext) -> TestSupportResult<()> {
    durable_shell.with_visual(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |phase0_shell, _view_cx| {
                phase0_shell
                    .begin_document_command_group_for_tests()
                    .map_err(|error| grouping_error("begin group", &error))
            })
        })?;
        visual_cx.run_until_parked();
        Ok(())
    })
}

/// Convert a group-transition error into a scenario expectation failure.
fn grouping_error(context: &str, error: &HistoryError) -> TestSupportError {
    TestSupportError::expectation(format!("{context} failed: {error}"))
}

/// Assert that the recorded history error matches the expected error.
fn expect_history_error(expected: HistoryError, context: &str) -> TestSupportResult<()> {
    let actual = with_state(|state| state.error.clone());
    if actual != Some(expected) {
        return Err(TestSupportError::expectation(format!(
            "expected {context}; got {actual:?}"
        )));
    }
    Ok(())
}

/// Open a test shell and record its initial history length.
fn fresh_shell(cx: &mut TestAppContext) -> TestSupportResult<DurableShell> {
    reset_state();
    let shell = DurableShell::open_for_tests(cx);
    let history_before =
        shell.with_visual(cx, |visual_cx, view| Ok(read_history_len(visual_cx, view)))?;
    with_state(|state| {
        state.shell = Some(shell.clone());
        state.history_before = Some(history_before);
    });
    Ok(shell)
}

/// Prepare a test shell containing one shape and its baseline geometry.
#[given("a fresh Phase 0 shell test window with one shape")]
fn fresh_shell_with_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shell = fresh_shell(cx)?;
    let shape = shape_id(42);
    let (history_before, document_before, anchor_before) =
        shell.with_visual(cx, |visual_cx, view| {
            replace_document_for_grouping_test(visual_cx, view, document_with_one_shape(shape));
            let document = read_document(visual_cx, view);
            let anchor_before = first_anchor_for_shape(&document, shape, "before grouped command")?;
            Ok((read_history_len(visual_cx, view), document, anchor_before))
        })?;
    with_state(|state| {
        state.shape = Some(shape);
        state.history_before = Some(history_before);
        state.document_before = Some(document_before);
        state.anchor_before = Some(anchor_before);
    });
    Ok(())
}

/// Commit two moves as one grouped document command.
#[when("two shape moves are committed in one document command group")]
fn commit_two_grouped_moves(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shape = with_state(|state| state.shape).ok_or_else(|| missing("shape"))?;
    let history_before = history_before()?;
    shell()?.with_visual(cx, |visual_cx, view| {
        apply_grouped_moves_for_test(
            visual_cx,
            view,
            GroupedMovePlan {
                shape,
                history_before,
                first_delta: Vec2::new(2.0, 0.0),
                second_delta: Vec2::new(0.0, 3.0),
            },
        )?;
        visual_cx.update(|_window, app| {
            view.update(app, |phase0_shell, _view_cx| {
                phase0_shell
                    .end_document_command_group_for_tests()
                    .map_err(|error| grouping_error("end group", &error))
            })
        })?;
        visual_cx.run_until_parked();
        Ok(())
    })
}

/// Assert the grouped command produced the requested history increment.
#[then("the document history has gained {count:u64} entry")]
fn history_gained(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: u64,
) -> Result<(), TestSupportError> {
    let expected = history_before()?
        + usize::try_from(count).map_err(|error| {
            TestSupportError::expectation(format!("history increment is invalid: {error}"))
        })?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = read_history_len(visual_cx, view);
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "expected one realized history entry after grouped command commit; expected {expected}, got {actual}"
            )));
        }
        Ok(())
    })
}

/// Assert the shape reflects both moves committed in the group.
#[then("the shape reflects both grouped moves")]
fn shape_reflects_grouped_moves(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shape = with_state(|state| state.shape).ok_or_else(|| missing("shape"))?;
    let before = with_state(|state| state.anchor_before)
        .ok_or_else(|| missing("anchor before grouped moves"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let after = first_anchor_for_shape(&document, shape, "after grouped command")?;
        common::assert_vec2_close(
            after,
            before.add(Vec2::new(2.0, 0.0)).add(Vec2::new(0.0, 3.0)),
            "after grouped command",
        )
    })
}

/// Undo the grouped document command.
#[when("the last document change is undone")]
fn undo_last_document_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

/// Assert undo restores the shape's pre-group position.
#[then("the shape returns to its position before the grouped moves")]
fn shape_returns_before_grouped_moves(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shape = with_state(|state| state.shape).ok_or_else(|| missing("shape"))?;
    let before = with_state(|state| state.anchor_before)
        .ok_or_else(|| missing("anchor before grouped moves"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let after = first_anchor_for_shape(&document, shape, "after grouped undo")?;
        common::assert_vec2_close(after, before, "after grouped undo")
    })
}

/// Prepare a fresh test shell for invalid group-transition scenarios.
#[given("a fresh Phase 0 shell test window")]
fn fresh_test_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    fresh_shell(cx).map(|_| ())
}

/// Attempt to end a document command group that was never begun.
#[when("a document command group is ended without being begun")]
fn end_group_without_begin(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let result = shell()?.with_visual(cx, |visual_cx, view| {
        Ok(visual_cx.update(|_window, app| {
            view.update(app, |phase0_shell, _view_cx| {
                phase0_shell.end_document_command_group_for_tests()
            })
        }))
    })?;
    with_state(|state| state.error = result.err());
    Ok(())
}

/// Assert the invalid end operation reports no active group.
#[then("the history error is no active group")]
fn history_error_is_no_active_group() -> Result<(), TestSupportError> {
    expect_history_error(HistoryError::NoActiveGroup, "no active group error")
}

/// Assert the invalid group transition left history unchanged.
#[then("the document history is unchanged")]
fn history_is_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = history_before()?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = read_history_len(visual_cx, view);
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "expected failed grouping boundary call to leave history unchanged; expected {expected}, got {actual}"
            )));
        }
        Ok(())
    })
}

/// Prepare a test shell with an active document command group.
#[given("a fresh Phase 0 shell test window with an active document command group")]
fn fresh_shell_with_active_group(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shell = fresh_shell(cx)?;
    begin_group(&shell, cx)
}

/// Attempt to begin a nested document command group.
#[when("another document command group is begun")]
fn begin_nested_group(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let result = shell()?.with_visual(cx, |visual_cx, view| {
        Ok(visual_cx.update(|_window, app| {
            view.update(app, |phase0_shell, _view_cx| {
                phase0_shell.begin_document_command_group_for_tests()
            })
        }))
    })?;
    with_state(|state| state.error = result.err());
    Ok(())
}

/// Assert the nested group attempt reports an active group.
#[then("the history error is group already active")]
fn history_error_is_group_already_active() -> Result<(), TestSupportError> {
    expect_history_error(
        HistoryError::GroupAlreadyActive,
        "group already active error",
    )
}

/// Assert the original active group can still be closed.
#[then("the active document command group remains closable")]
fn active_group_remains_closable(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |phase0_shell, _view_cx| {
                phase0_shell
                    .end_document_command_group_for_tests()
                    .map_err(|error| grouping_error("close active group", &error))
            })
        })?;
        visual_cx.run_until_parked();
        Ok(())
    })
}

/// Run the successful grouped-command feature scenario.
#[scenario(
    path = "tests/features/history_command_grouping_undo.feature",
    name = "Grouped document commands collapse to one undo step",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn grouped_commands(#[from(state_cleanup)] _cleanup: StateCleanup) {}

/// Run the invalid end-without-begin feature scenario.
#[scenario(
    path = "tests/features/history_command_grouping_undo.feature",
    name = "Ending a group without beginning one preserves history",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn end_without_begin(#[from(state_cleanup)] _cleanup: StateCleanup) {}

/// Run the nested-group rejection feature scenario.
#[scenario(
    path = "tests/features/history_command_grouping_undo.feature",
    name = "Beginning a nested group preserves history",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn nested_group(#[from(state_cleanup)] _cleanup: StateCleanup) {}
