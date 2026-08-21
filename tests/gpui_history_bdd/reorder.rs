//! BDD step bindings for shape reorder history.
//!
//! The steps select overlapping shapes, lower or raise the selected shape,
//! and verify ordering, stable identifiers, and undo history for the feature
//! scenario. The parent integration binary runs it with `GpuiHarness`, while
//! shared canvas, document, history, and durable-shell utilities support the
//! individual steps.

use gpui::TestAppContext;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenario, then, when};
use serial_test::serial;

use super::*;
use crate::history_bdd_support::{DurableShell, missing};

#[derive(Default, ScenarioState)]
struct ReorderState {
    shell: Slot<DurableShell>,
    click_point: Slot<gpui::Point<gpui::Pixels>>,
    lower: Slot<ShapeId>,
    higher: Slot<ShapeId>,
    expected_ids: Slot<Vec<ShapeId>>,
    history_before: Slot<usize>,
}

crate::scenario_state!(ReorderState);

/// Retrieve the durable shell stored in scenario state.
fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.get()).ok_or_else(|| missing("Phase 0 shell"))
}

/// Retrieve the lower and higher shape identifiers from scenario state.
fn shape_pair() -> Result<(ShapeId, ShapeId), TestSupportError> {
    let (lower, higher) = with_state(|state| (state.lower.get(), state.higher.get()));
    Ok((
        lower.ok_or_else(|| missing("lower shape"))?,
        higher.ok_or_else(|| missing("higher shape"))?,
    ))
}

/// Prepare overlapping shapes and record their expected initial order.
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
        state.shell.set(shell);
        state.click_point.set(points.start);
        state.lower.set(lower);
        state.higher.set(higher);
        state.expected_ids.set(expected_ids);
        state.history_before.set(history_before);
    });
    Ok(())
}

/// Click the overlap and verify the topmost shape is selected.
#[when("the overlap is clicked")]
fn click_overlap(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let point = with_state(|state| state.click_point.get())
        .ok_or_else(|| missing("overlap click point"))?;
    let (_, higher) = shape_pair()?;
    shell()?.with_visual(cx, |visual_cx, view| {
        click_and_verify_topmost(visual_cx, view, point, higher)
    })
}

/// Assert the topmost shape remains selected after the click.
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

/// Lower the selected shape with the reorder shortcut.
#[when("the selected shape is lowered")]
fn lower_selected_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_key(visual_cx, "[", Modifiers::secondary_key());
        Ok(())
    })
}

/// Raise the selected shape with the reorder shortcut.
#[when("the selected shape is raised")]
fn raise_selected_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_key(visual_cx, "]", Modifiers::secondary_key());
        Ok(())
    })
}

/// Assert reordering preserves the shape identifiers.
#[then("the shape identifiers are unchanged")]
fn shape_identifiers_are_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| state.expected_ids.get())
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

/// Assert the selected shape's relative order against its peer.
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

/// Assert the selected shape is below the other shape.
#[then("the selected shape is below the other drawn shape")]
fn selected_shape_is_below(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_selected_shape_order(cx, false)
}

/// Assert the selected shape is above the other shape.
#[then("the selected shape is above the other drawn shape")]
fn selected_shape_is_above(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_selected_shape_order(cx, true)
}

/// Assert the document history increased by the requested count.
#[then("the document history has gained {count:u64} entry")]
fn history_has_gained(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: u64,
) -> Result<(), TestSupportError> {
    let before = with_state(|state| state.history_before.get())
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

/// Assert the plural history-entry form using the shared count check.
#[then("the document history has gained {count:u64} entries")]
fn history_has_gained_entries(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: u64,
) -> Result<(), TestSupportError> {
    history_has_gained(cx, count)
}

/// Undo the most recent reorder document change.
#[when("the last document change is undone")]
fn undo_last_document_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual(cx, |visual_cx, _view| {
        common::simulate_document_undo(visual_cx);
        Ok(())
    })
}

/// Run the shape reorder undo feature scenario.
#[scenario(
    path = "tests/features/history_reorder_undo.feature",
    name = "Lowering and raising shapes are undoable",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn reorder_history(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
