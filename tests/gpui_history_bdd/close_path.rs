//! BDD step bindings for closing and reopening a path through history.
//!
//! The steps draw an open three-anchor path, close it at the first anchor,
//! and verify that undo restores the open state. The parent integration binary
//! executes the feature scenario with `GpuiHarness`; shared canvas, document,
//! history, and durable-shell utilities are imported from the test support
//! modules.

use std::cell::RefCell;

use gauss::model::Vec2;
use gpui::{Bounds, Pixels, Point, TestAppContext};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[derive(Default)]
struct ClosePathState {
    shell: Option<DurableShell>,
    bounds: Option<Bounds<Pixels>>,
    first_point: Option<Point<Pixels>>,
    first_anchor_pos: Option<Vec2>,
    initial_history_len: Option<usize>,
}

thread_local! {
    static STATE: RefCell<ClosePathState> = RefCell::new(ClosePathState::default());
}

/// Apply a closure to the close-path scenario state.
fn with_state<R>(f: impl FnOnce(&mut ClosePathState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

/// Reset all close-path state before or after a scenario.
fn reset_state() {
    with_state(|state| *state = ClosePathState::default());
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

/// Prepare an open three-anchor path and record its initial state.
#[given("a fresh Phase 0 shell window with an open three-anchor path")]
fn fresh_shell_with_open_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (bounds, first_point, first_anchor_pos, initial_history_len) =
        shell.with_visual(cx, |visual_cx, view| {
            let bounds = canvas_bounds(visual_cx)?;
            let (p1, p2, p3) = triangle_points(&bounds);
            draw_point(visual_cx, p1);
            draw_point(visual_cx, p2);
            draw_point(visual_cx, p3);
            let document = read_document(visual_cx, view);
            let shape = require_draw_shape(&document, "before close")?;
            if shape.path.closed {
                return Err(TestSupportError::expectation(
                    "expected path to be open before close",
                ));
            }
            if shape.path.anchors.len() != 3 {
                return Err(TestSupportError::expectation(format!(
                    "expected three anchors before close, got {}",
                    shape.path.anchors.len()
                )));
            }
            let first_anchor_pos = shape
                .path
                .anchors
                .first()
                .ok_or_else(|| missing("anchor 0"))?
                .pos;
            Ok((
                bounds,
                p1,
                first_anchor_pos,
                read_history_len(visual_cx, view),
            ))
        })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.bounds = Some(bounds);
        state.first_point = Some(first_point);
        state.first_anchor_pos = Some(first_anchor_pos);
        state.initial_history_len = Some(initial_history_len);
    });
    Ok(())
}

/// Close the path by clicking the first anchor.
#[when("the path is closed by clicking its first anchor")]
fn close_path_at_first_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (maybe_bounds, maybe_first_point, maybe_anchor_pos) =
        with_state(|state| (state.bounds, state.first_point, state.first_anchor_pos));
    let canvas_bounds = maybe_bounds.ok_or_else(|| missing("canvas bounds"))?;
    let initial_point = maybe_first_point.ok_or_else(|| missing("first canvas point"))?;
    let first_anchor_position = maybe_anchor_pos.ok_or_else(|| missing("first anchor position"))?;
    shell()?.with_visual(cx, |visual_cx, _view| {
        let close_point =
            anchor_to_canvas_point(&canvas_bounds, first_anchor_position, initial_point);
        draw_point(visual_cx, close_point);
        Ok(())
    })
}

/// Undo the close-path document change.
#[when("the last document change is undone")]
fn undo_last_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

/// Assert the path's closed flag and anchor count.
fn assert_path_state(
    cx: &mut TestAppContext,
    expected_closed: bool,
    expected_anchors: usize,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after close-path operation")?;
        if shape.path.closed != expected_closed {
            return Err(TestSupportError::expectation(format!(
                "expected path closed state {expected_closed}, got {}",
                shape.path.closed
            )));
        }
        if shape.path.anchors.len() != expected_anchors {
            return Err(TestSupportError::expectation(format!(
                "expected {expected_anchors} anchors, got {}",
                shape.path.anchors.len()
            )));
        }
        Ok(())
    })
}

/// Assert that the path is closed with the expected number of anchors.
#[then("the path is closed with {anchors:usize} anchors")]
fn path_is_closed(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    anchors: usize,
) -> Result<(), TestSupportError> {
    assert_path_state(cx, true, anchors)
}

/// Assert that the path is open with the expected number of anchors.
#[then("the path is open with {anchors:usize} anchors")]
fn path_is_open(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    anchors: usize,
) -> Result<(), TestSupportError> {
    assert_path_state(cx, false, anchors)
}

/// Assert the document history increased by the requested number of entries.
#[then("the document history has gained {entries:usize} entry")]
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

/// Run the close-path undo and reopen feature scenario.
#[scenario(
    path = "tests/features/history_close_path_undo.feature",
    name = "Closing a path creates one undo entry and undo reopens it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn close_path_history_scenario(#[from(state_cleanup)] _cleanup: StateCleanup) {}
