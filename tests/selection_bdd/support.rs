//! Shared GPUI lifecycle support for the selection BDD integration binaries.
//!
//! Each binary binds scenarios from `selection.feature` through `GpuiHarness`
//! and textually includes this module. The support retains window handles,
//! interaction points, and binary-specific payloads between steps. `common`
//! initializes the Phase 0 shell and owns GPUI interactions and coordinates;
//! model-only assertions live in `test_support::selection` for reuse outside
//! the GPUI harness.

use std::{any::Any, cell::RefCell, fmt};

use gauss::ui::Phase0Shell;
use gpui::{
    AnyWindowHandle, Entity, Pixels, Point, TestAppContext, VisualContext, VisualTestContext,
};
use rstest::fixture;
use rstest_bdd_macros::given;
use test_support::{TestSupportError, TestSupportResult};

use crate::common::{ensure_initial_draw, init_test_app};

/// Typed descriptions of selection BDD step data and interaction points.
///
/// Each variant preserves the context text emitted by the original scenario
/// steps while preventing callers from supplying unrelated raw strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioContext {
    /// The start point of an unselected drag.
    UnselectedDragStart,
    /// The end point of an unselected drag.
    UnselectedDragEnd,
    /// The recorded unselected drag state.
    UnselectedDrag,
    /// A selected square.
    SelectedSquare,
    /// The drag state recorded after a press.
    DragStateAfterPress,
    /// A square expected to remain unchanged.
    UnchangedSquare,
    /// A point on the empty canvas.
    EmptyCanvasPoint,
    /// A selection anchor click.
    AnchorClick,
    /// The selection captured from an anchor click.
    AnchorSelection,
    /// The drag state captured from a Shift-click.
    ShiftClickDragState,
    /// The press point for a selected square.
    SelectedSquarePress,
    /// The drag end point for a selected square.
    SelectedSquareDragEnd,
    /// The selected squares.
    SelectedSquares,
    /// The snapshot taken before a multi-shape drag.
    MultiShapeDragSnapshot,
    /// The centre point of a square.
    SquareCentre,
    /// The selection recorded from a bounding-box press.
    BoundingBoxPress,
    /// The selection produced by bounding-box selection.
    BoundingBoxSelection,
    /// A right-click point.
    RightClickPoint,
    /// A point for a zero-delta drag.
    ZeroDeltaDragPoint,
    /// A snapshot of the selection.
    SelectionSnapshot,
    /// The document history length.
    HistoryLength,
}

impl fmt::Display for ScenarioContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let context = match self {
            Self::UnselectedDragStart => "unselected drag start",
            Self::UnselectedDragEnd => "unselected drag end",
            Self::UnselectedDrag => "unselected drag",
            Self::SelectedSquare => "selected square",
            Self::DragStateAfterPress => "drag state after press",
            Self::UnchangedSquare => "unchanged square",
            Self::EmptyCanvasPoint => "empty canvas point",
            Self::AnchorClick => "anchor click",
            Self::AnchorSelection => "anchor selection",
            Self::ShiftClickDragState => "Shift-click drag state",
            Self::SelectedSquarePress => "selected square press",
            Self::SelectedSquareDragEnd => "selected square drag end",
            Self::SelectedSquares => "selected squares",
            Self::MultiShapeDragSnapshot => "multi-shape drag snapshot",
            Self::SquareCentre => "square centre",
            Self::BoundingBoxPress => "bounding-box press",
            Self::BoundingBoxSelection => "bounding-box selection",
            Self::RightClickPoint => "right-click point",
            Self::ZeroDeltaDragPoint => "zero-delta drag point",
            Self::SelectionSnapshot => "selection snapshot",
            Self::HistoryLength => "history length",
        };
        formatter.write_str(context)
    }
}

/// Typed descriptions of presses that must not start a selection drag.
///
/// The variants retain the press-specific diagnostics used by the Gherkin
/// selection scenarios.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoDragPress {
    /// A bounding-box press before a square has been selected.
    UnselectedBoundingBox,
    /// The Shift-click that adds a selection anchor.
    AdditiveShiftClick,
    /// The Shift-click that toggles a selection anchor.
    ToggleShiftClick,
}

impl NoDragPress {
    /// Return the diagnostic for a press that unexpectedly started a drag.
    #[must_use]
    pub const fn drag_started_message(self) -> &'static str {
        match self {
            Self::UnselectedBoundingBox => "unselected bounding-box press started a drag",
            Self::AdditiveShiftClick => "additive Shift-click started a drag gesture",
            Self::ToggleShiftClick => "toggle Shift-click started a drag gesture",
        }
    }

    /// Return the context used when the expected press state was not recorded.
    #[must_use]
    pub const fn missing_record_message(self) -> &'static str {
        match self {
            Self::UnselectedBoundingBox => "recorded by the drag step",
            Self::AdditiveShiftClick => "recorded by the second-anchor Shift-click step",
            Self::ToggleShiftClick => "recorded by the first-anchor Shift-click step",
        }
    }
}

/// Durable handles and interaction points shared by selection scenario steps.
#[derive(Default)]
pub struct ScenarioState {
    entity: Option<Entity<Phase0Shell>>,
    window: Option<AnyWindowHandle>,
    /// Screen points shared between arrangement and interaction steps.
    pub points: Vec<Point<Pixels>>,
    data: Option<Box<dyn Any>>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

/// Mutate the lifecycle state for the current scenario.
pub fn with_state<R>(f: impl FnOnce(&mut ScenarioState) -> R) -> R {
    SCENARIO_STATE.with(|cell| f(&mut cell.borrow_mut()))
}

/// Replace the scenario-specific payload.
pub fn set_scenario_data<T: 'static>(data: T) {
    with_state(|state| state.data = Some(Box::new(data)));
}

fn with_typed_scenario_data<R>(
    context: ScenarioContext,
    lookup: impl FnOnce(&mut dyn Any) -> Option<R>,
) -> TestSupportResult<R> {
    with_state(|state| state.data.as_deref_mut().and_then(lookup))
        .ok_or_else(|| TestSupportError::missing("scenario data", context.to_string()))
}

/// Read the scenario-specific payload for a typed scenario context.
///
/// # Errors
///
/// Returns an error when the scenario payload is absent or has a different
/// concrete type.
pub fn with_scenario_data<T: 'static, R>(
    context: ScenarioContext,
    f: impl FnOnce(&T) -> R,
) -> TestSupportResult<R> {
    with_typed_scenario_data(context, |data| data.downcast_ref::<T>().map(f))
}

/// Mutate the scenario-specific payload for a typed scenario context.
///
/// # Errors
///
/// Returns an error when the scenario payload is absent or has a different
/// concrete type.
pub fn with_mut_scenario_data<T: 'static, R>(
    context: ScenarioContext,
    f: impl FnOnce(&mut T) -> R,
) -> TestSupportResult<R> {
    with_typed_scenario_data(context, |data| data.downcast_mut::<T>().map(f))
}

fn reset_state_after_scenario() {
    SCENARIO_STATE.with(|cell| *cell.borrow_mut() = ScenarioState::default());
}

fn reset_state_before_assignment() {
    reset_state_after_scenario();
}

/// Drop guard that clears thread-local scenario state after a test.
pub struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state_after_scenario();
    }
}

/// Reset scenario state before execution and return its cleanup guard.
#[fixture]
pub fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

/// Require a typed recorded press not to have started a drag gesture.
///
/// # Errors
///
/// Returns an error when the press started a drag or no press state was
/// recorded.
pub fn assert_no_drag_after_press(
    drag_started_after_press: Option<bool>,
    press: NoDragPress,
) -> TestSupportResult<()> {
    match drag_started_after_press {
        Some(false) => Ok(()),
        Some(true) => Err(TestSupportError::expectation(
            press.drag_started_message().to_owned(),
        )),
        None => Err(TestSupportError::missing(
            "drag state after press",
            press.missing_record_message(),
        )),
    }
}

/// Reconstruct a visual context from the durable handles for this scenario.
///
/// # Errors
///
/// Returns an error when the scenario handles are absent or the supplied
/// operation fails.
pub fn with_visual_cx<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
) -> TestSupportResult<R> {
    let handles = with_state(|state| (state.entity.clone(), state.window));
    let (Some(entity), Some(window)) = handles else {
        return Err(TestSupportError::missing(
            "scenario handles",
            "set by the fresh-window step",
        ));
    };
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    f(&mut visual_cx, &entity)
}

/// Read a recorded screen point by index for a typed scenario context.
///
/// # Errors
///
/// Returns an error when no point is recorded at `index`.
pub fn require_point(index: usize, context: ScenarioContext) -> TestSupportResult<Point<Pixels>> {
    with_state(|state| state.points.get(index).copied()).ok_or_else(|| {
        TestSupportError::missing(format!("scenario point {index}"), context.to_string())
    })
}

#[cfg(test)]
mod tests {
    //! Focused checks for the typed selection BDD support diagnostics.

    use super::{NoDragPress, ScenarioContext, assert_no_drag_after_press};

    #[test]
    fn scenario_context_renders_existing_context_text() {
        for (context, expected) in [
            (
                ScenarioContext::UnselectedDragStart,
                "unselected drag start",
            ),
            (ScenarioContext::UnselectedDragEnd, "unselected drag end"),
            (ScenarioContext::UnselectedDrag, "unselected drag"),
            (ScenarioContext::SelectedSquare, "selected square"),
            (
                ScenarioContext::DragStateAfterPress,
                "drag state after press",
            ),
            (ScenarioContext::UnchangedSquare, "unchanged square"),
            (ScenarioContext::EmptyCanvasPoint, "empty canvas point"),
            (ScenarioContext::AnchorClick, "anchor click"),
            (ScenarioContext::AnchorSelection, "anchor selection"),
            (
                ScenarioContext::ShiftClickDragState,
                "Shift-click drag state",
            ),
            (
                ScenarioContext::SelectedSquarePress,
                "selected square press",
            ),
            (
                ScenarioContext::SelectedSquareDragEnd,
                "selected square drag end",
            ),
            (ScenarioContext::SelectedSquares, "selected squares"),
            (
                ScenarioContext::MultiShapeDragSnapshot,
                "multi-shape drag snapshot",
            ),
            (ScenarioContext::SquareCentre, "square centre"),
            (ScenarioContext::BoundingBoxPress, "bounding-box press"),
            (
                ScenarioContext::BoundingBoxSelection,
                "bounding-box selection",
            ),
            (ScenarioContext::RightClickPoint, "right-click point"),
            (ScenarioContext::ZeroDeltaDragPoint, "zero-delta drag point"),
            (ScenarioContext::SelectionSnapshot, "selection snapshot"),
            (ScenarioContext::HistoryLength, "history length"),
        ] {
            assert_eq!(
                context.to_string(),
                expected,
                "{context:?} must preserve its scenario error context"
            );
        }
    }

    #[test]
    fn no_drag_press_preserves_existing_error_messages() {
        for (press, drag_started_message, missing_record_message) in [
            (
                NoDragPress::UnselectedBoundingBox,
                "unselected bounding-box press started a drag",
                "recorded by the drag step",
            ),
            (
                NoDragPress::AdditiveShiftClick,
                "additive Shift-click started a drag gesture",
                "recorded by the second-anchor Shift-click step",
            ),
            (
                NoDragPress::ToggleShiftClick,
                "toggle Shift-click started a drag gesture",
                "recorded by the first-anchor Shift-click step",
            ),
        ] {
            let drag_error = assert_no_drag_after_press(Some(true), press)
                .expect_err("a started drag must fail the no-drag assertion");
            assert_eq!(
                drag_error.to_string(),
                format!("expectation failed: {drag_started_message}"),
                "{press:?} must preserve its drag-start diagnostic"
            );

            let missing_error = assert_no_drag_after_press(None, press)
                .expect_err("an absent press state must fail the no-drag assertion");
            assert_eq!(
                missing_error.to_string(),
                format!("missing drag state after press: {missing_record_message}"),
                "{press:?} must preserve its missing-state diagnostic"
            );
        }
    }
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    reset_state_before_assignment();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    with_state(|state| {
        state.entity = Some(entity);
        state.window = Some(window);
    });
}
