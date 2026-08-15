//! BDD bindings for Bezier-handle drag history.

use std::cell::RefCell;

use gpui::{Modifiers, MouseButton, TestAppContext};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[derive(Default)]
struct DragHandleState {
    shell: Option<DurableShell>,
    setup: Option<HandleDragSetup>,
    initial_history_len: Option<usize>,
}

thread_local! {
    static STATE: RefCell<DragHandleState> = RefCell::new(DragHandleState::default());
}

fn with_state<R>(f: impl FnOnce(&mut DragHandleState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = DragHandleState::default());
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

#[given("a fresh Phase 0 shell window with a selected Bezier handle")]
fn fresh_shell_with_handle(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (setup, initial_history_len) = shell.with_visual(cx, |visual_cx, view| {
        let setup = setup_handle_drag(visual_cx, view)?;
        Ok((setup, read_history_len(visual_cx, view)))
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.setup = Some(setup);
        state.initial_history_len = Some(initial_history_len);
    });
    Ok(())
}

#[when("the selected Bezier handle is dragged")]
fn drag_selected_handle(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let setup = with_state(|state| state.setup).ok_or_else(|| missing("handle drag setup"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        visual_cx.simulate_mouse_down(setup.handle_start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        assert_handle_selection_includes_shape(visual_cx, view, setup.shape_id)?;
        visual_cx.simulate_mouse_move(setup.handle_end, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(setup.handle_end, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
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

#[then("the anchor stays fixed and its outgoing handle moves by the drag delta")]
fn anchor_fixed_handle_moved(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let setup = with_state(|state| state.setup).ok_or_else(|| missing("handle drag setup"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after dragging handle")?;
        let anchor = shape
            .path
            .anchors
            .first()
            .ok_or_else(|| missing("first anchor after drag"))?;
        assert_vec2_close(
            anchor.pos,
            setup.first_anchor_pos,
            "anchor position stable when dragging handle",
        )?;
        let handle_out = anchor
            .handle_out
            .ok_or_else(|| missing("handle_out after dragging"))?;
        assert_vec2_close(
            handle_out,
            setup.original_handle_out.add(setup.scenario.delta),
            "handle_out moved by delta",
        )
    })
}

#[then("the anchor and outgoing handle return to their positions before the drag")]
fn anchor_and_handle_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let setup = with_state(|state| state.setup).ok_or_else(|| missing("handle drag setup"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after undo")?;
        let anchor = shape
            .path
            .anchors
            .first()
            .ok_or_else(|| missing("first anchor after undo"))?;
        let handle_out = anchor
            .handle_out
            .ok_or_else(|| missing("handle_out after undo"))?;
        assert_vec2_close(anchor.pos, setup.first_anchor_pos, "anchor restored")?;
        assert_vec2_close(handle_out, setup.original_handle_out, "handle_out restored")
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
    path = "tests/features/history_drag_handle_undo.feature",
    name = "Dragging a handle creates one undo entry and undo restores it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn drag_handle_history_scenario(#[from(state_cleanup)] _cleanup: StateCleanup) {}
