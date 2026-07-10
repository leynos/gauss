//! BDD bindings for anchor insertion and deletion history.

use std::cell::RefCell;

use gpui::TestAppContext;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[derive(Default)]
struct AnchorEditState {
    shell: Option<DurableShell>,
    setup: Option<DrawnPathSetup>,
    midpoint: Option<Vec2>,
    initial_history_len: Option<usize>,
}

thread_local! {
    static STATE: RefCell<AnchorEditState> = RefCell::new(AnchorEditState::default());
}

fn with_state<R>(f: impl FnOnce(&mut AnchorEditState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = AnchorEditState::default());
}

struct StateCleanup;

impl Drop for StateCleanup {
    fn drop(&mut self) {
        reset_state();
    }
}

#[fixture]
fn state_cleanup() -> StateCleanup {
    reset_state();
    StateCleanup
}

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| missing("Phase 0 shell"))
}

#[given("a fresh Phase 0 shell window with a two-anchor path")]
fn fresh_shell_with_two_anchor_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (setup, midpoint, initial_history_len) = shell.with_visual(cx, |visual_cx, view| {
        let setup = draw_two_point_path(visual_cx, view)?;
        simulate_escape(visual_cx);
        let midpoint = Vec2::new(
            math::midpoint(setup.start_pos.x, setup.end_pos.x),
            math::midpoint(setup.start_pos.y, setup.end_pos.y),
        );
        Ok((setup, midpoint, read_history_len(visual_cx, view)))
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.setup = Some(setup);
        state.midpoint = Some(midpoint);
        state.initial_history_len = Some(initial_history_len);
    });
    Ok(())
}

#[when("an anchor is inserted at the path midpoint")]
fn insert_midpoint_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (maybe_setup, maybe_midpoint) = with_state(|state| (state.setup, state.midpoint));
    let path_setup = maybe_setup.ok_or_else(|| missing("drawn path setup"))?;
    let path_midpoint = maybe_midpoint.ok_or_else(|| missing("path midpoint"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        insert_anchor_at_midpoint(visual_cx, view, &path_setup, path_midpoint)
    })
}

#[when("the inserted anchor is deleted")]
fn delete_midpoint_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, delete_selected_anchor)
}

#[when("the last document change is undone")]
fn undo_last_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[when("the last document change is redone")]
fn redo_last_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_document_redo(visual_cx);
        Ok(())
    })
}

#[then("the path has {anchors:usize} anchors and {segments:usize} segments")]
fn path_has_counts(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    anchors: usize,
    segments: usize,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        read_shape_with_counts(
            visual_cx,
            view,
            PathCounts { anchors, segments },
            "after history operation",
        )?;
        Ok(())
    })
}

#[then("the path has {anchors:usize} anchors and 1 segment")]
fn path_has_one_segment(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    anchors: usize,
) -> Result<(), TestSupportError> {
    path_has_counts(cx, anchors, 1)
}

#[then("the document history has gained {entries:usize} entries")]
fn history_has_gained_entries(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    entries: usize,
) -> Result<(), TestSupportError> {
    let initial = with_state(|state| state.initial_history_len)
        .ok_or_else(|| missing("initial history length"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = read_history_len(visual_cx, view);
        if actual != initial + entries {
            return Err(TestSupportError::expectation(format!(
                "expected history length {}, got {actual}",
                initial + entries
            )));
        }
        Ok(())
    })
}

#[then("the document history has gained {entries:usize} entry")]
fn history_has_gained_entry(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    entries: usize,
) -> Result<(), TestSupportError> {
    history_has_gained_entries(cx, entries)
}

#[scenario(
    path = "tests/features/history_anchor_edit_undo.feature",
    name = "Anchor insertion and deletion round-trip through undo and redo",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn anchor_edit_history_scenario(#[from(state_cleanup)] _cleanup: StateCleanup) {}
