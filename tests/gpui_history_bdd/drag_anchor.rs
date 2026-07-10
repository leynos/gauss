//! BDD bindings for anchor-drag history.

use std::cell::RefCell;

use gpui::TestAppContext;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[derive(Default)]
struct DragAnchorState {
    shell: Option<DurableShell>,
    scenario: Option<CanvasDragScenario>,
    shape_id: Option<ShapeId>,
    original_anchors: Option<(Anchor, Anchor)>,
    initial_history_len: Option<usize>,
}

thread_local! {
    static STATE: RefCell<DragAnchorState> = RefCell::new(DragAnchorState::default());
}

fn with_state<R>(f: impl FnOnce(&mut DragAnchorState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = DragAnchorState::default());
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

#[given("a fresh Phase 0 shell window with a two-anchor line selected for editing")]
fn fresh_shell_with_line(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (scenario, shape_id, original_anchors, initial_history_len) =
        shell.with_visual(cx, |visual_cx, view| {
            let scenario = canvas_drag_scenario(visual_cx, 24.0, 12.0)?;
            draw_two_point_line_path(visual_cx, scenario);
            let document = read_document(visual_cx, view);
            let shape = require_draw_shape(&document, "after drawing two points")?.clone();
            let original_anchors = first_two_anchors(&shape)?;
            simulate_escape(visual_cx);
            Ok((
                scenario,
                shape.id,
                original_anchors,
                read_history_len(visual_cx, view),
            ))
        })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.scenario = Some(scenario);
        state.shape_id = Some(shape_id);
        state.original_anchors = Some(original_anchors);
        state.initial_history_len = Some(initial_history_len);
    });
    Ok(())
}

#[when("the first anchor is dragged")]
fn drag_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (maybe_scenario, maybe_shape_id) = with_state(|state| (state.scenario, state.shape_id));
    let drag_scenario = maybe_scenario.ok_or_else(|| missing("canvas drag scenario"))?;
    let drawn_shape_id = maybe_shape_id.ok_or_else(|| missing("drawn shape id"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        drag_first_anchor(visual_cx, view, drawn_shape_id, drag_scenario)
    })
}

#[when("the last document change is undone")]
fn undo_last_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[then("only the first anchor moves by the drag delta")]
fn only_first_anchor_moves(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (maybe_scenario, maybe_originals) =
        with_state(|state| (state.scenario, state.original_anchors.clone()));
    let drag_scenario = maybe_scenario.ok_or_else(|| missing("canvas drag scenario"))?;
    let original_anchors = maybe_originals.ok_or_else(|| missing("original anchors"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after dragging anchor")?;
        verify_anchor_moved(shape, &original_anchors, drag_scenario.delta)
    })
}

#[then("both anchors return to their positions before the drag")]
fn anchors_are_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let originals = with_state(|state| state.original_anchors.clone())
        .ok_or_else(|| missing("original anchors"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after undo")?;
        verify_anchor_restored(shape, &originals)
    })
}

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

#[scenario(
    path = "tests/features/history_drag_anchor_undo.feature",
    name = "Dragging an anchor creates one undo entry and undo restores it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn drag_anchor_history_scenario(#[from(state_cleanup)] _cleanup: StateCleanup) {}
