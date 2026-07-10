//! BDD bindings for shape reorder history.

use std::cell::RefCell;

use gpui::TestAppContext;
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[derive(Default)]
struct ReorderState {
    shell: Option<DurableShell>,
    click_point: Option<gpui::Point<gpui::Pixels>>,
    lower: Option<ShapeId>,
    higher: Option<ShapeId>,
    expected_ids: Option<Vec<ShapeId>>,
    history_before: Option<usize>,
}

thread_local! {
    static STATE: RefCell<ReorderState> = RefCell::new(ReorderState::default());
}

fn with_state<R>(f: impl FnOnce(&mut ReorderState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = ReorderState::default());
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

fn shape_pair() -> Result<(ShapeId, ShapeId), TestSupportError> {
    let (lower, higher) = with_state(|state| (state.lower, state.higher));
    Ok((
        lower.ok_or_else(|| missing("lower shape"))?,
        higher.ok_or_else(|| missing("higher shape"))?,
    ))
}

#[given("a fresh Phase 0 shell window with two overlapping drawn shapes")]
fn fresh_shell_with_overlapping_shapes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let shell = DurableShell::open(cx);
    let (points, lower, higher, expected_ids, history_before) =
        shell.with_visual(cx, |visual_cx, view| {
            let points = line_points(&common::canvas_bounds(visual_cx)?);
            draw_overlapping_lines(visual_cx, points);
            let document = read_document(visual_cx, view);
            let (lower, higher, expected_ids) = verify_initial_shapes_and_order(&document)?;
            Ok((
                points,
                lower,
                higher,
                expected_ids,
                common::read_history_len(visual_cx, view),
            ))
        })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.click_point = Some(points.start);
        state.lower = Some(lower);
        state.higher = Some(higher);
        state.expected_ids = Some(expected_ids);
        state.history_before = Some(history_before);
    });
    Ok(())
}

#[when("the overlap is clicked")]
fn click_overlap(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point =
        with_state(|state| state.click_point).ok_or_else(|| missing("overlap click point"))?;
    let (_, higher) = shape_pair()?;
    shell()?.with_visual(cx, |visual_cx, view| {
        click_and_verify_topmost(visual_cx, view, point, higher)
    })
}

#[then("the topmost shape is selected")]
fn topmost_shape_is_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (_, higher) = shape_pair()?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let selection = read_selection(visual_cx, view);
        let selected = selected_shape_id(&selection);
        if selected != Some(higher) {
            return Err(TestSupportError::expectation(format!(
                "expected the top-most shape {higher:?} to remain selected; got {selected:?}"
            )));
        }
        Ok(())
    })
}

#[when("the selected shape is lowered")]
fn lower_selected_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_key(visual_cx, "[", Modifiers::secondary_key());
        Ok(())
    })
}

#[when("the selected shape is raised")]
fn raise_selected_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_key(visual_cx, "]", Modifiers::secondary_key());
        Ok(())
    })
}

#[then("the shape identifiers are unchanged")]
fn shape_identifiers_are_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| state.expected_ids.clone())
        .ok_or_else(|| missing("initial shape identifiers"))?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = require_sorted_drawn_shape_ids(&read_document(visual_cx, view))?;
        if actual != expected {
            return Err(TestSupportError::expectation(
                "expected shape ids to remain stable after reordering",
            ));
        }
        Ok(())
    })
}

fn assert_selected_shape_order(
    cx: &mut TestAppContext,
    is_selected_above: bool,
) -> TestSupportResult<()> {
    let (lower, higher) = shape_pair()?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        if is_selected_above {
            assert_relative_order(&document, lower, higher, "selected shape should be above")
        } else {
            assert_relative_order(&document, higher, lower, "selected shape should be below")
        }
    })
}

#[then("the selected shape is below the other drawn shape")]
fn selected_shape_is_below(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_selected_shape_order(cx, false)
}

#[then("the selected shape is above the other drawn shape")]
fn selected_shape_is_above(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_selected_shape_order(cx, true)
}

#[then("the document history has gained {count:u64} entry")]
fn history_has_gained(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: u64,
) -> Result<(), TestSupportError> {
    let before = with_state(|state| state.history_before)
        .ok_or_else(|| missing("initial history length"))?;
    let increment = usize::try_from(count).map_err(|error| {
        TestSupportError::expectation(format!("history increment is invalid: {error}"))
    })?;
    shell()?.with_visual(cx, |visual_cx, view| {
        let actual = common::read_history_len(visual_cx, view);
        let expected = before + increment;
        if actual != expected {
            return Err(TestSupportError::expectation(format!(
                "expected document history length {expected}; got {actual}"
            )));
        }
        Ok(())
    })
}

#[then("the document history has gained {count:u64} entries")]
fn history_has_gained_entries(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: u64,
) -> Result<(), TestSupportError> {
    history_has_gained(cx, count)
}

#[when("the last document change is undone")]
fn undo_last_document_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[scenario(
    path = "tests/features/history_reorder_undo.feature",
    name = "Lowering and raising shapes are undoable",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn reorder_history(#[from(state_cleanup)] _cleanup: StateCleanup) {}
