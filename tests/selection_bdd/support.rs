//! Shared durable GPUI state for selection behavioural scenarios.

use std::{any::Any, cell::RefCell};

use gauss::ui::Phase0Shell;
use gpui::{
    AnyWindowHandle, Entity, Pixels, Point, TestAppContext, VisualContext, VisualTestContext,
};
use rstest::fixture;
use rstest_bdd_macros::given;
use test_support::{TestSupportError, TestSupportResult};

use crate::common::{ensure_initial_draw, init_test_app};

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

/// Mutate the scenario-specific payload as its concrete type.
pub fn with_scenario_data<T: 'static, R>(
    context: &str,
    f: impl FnOnce(&mut T) -> R,
) -> TestSupportResult<R> {
    with_state(|state| {
        state
            .data
            .as_deref_mut()
            .and_then(|data| data.downcast_mut::<T>())
            .map(f)
    })
    .ok_or_else(|| TestSupportError::missing("scenario data", context))
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

#[fixture]
/// Reset scenario state before execution and return its cleanup guard.
pub fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

/// Reconstruct a visual context from the durable handles for this scenario.
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

/// Read a recorded screen point by index.
pub fn require_point(index: usize, context: &str) -> TestSupportResult<Point<Pixels>> {
    with_state(|state| state.points.get(index).copied()).ok_or_else(|| {
        TestSupportError::missing(format!("scenario point {index}"), context.to_owned())
    })
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state_before_assignment();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    with_state(|state| {
        state.entity = Some(entity);
        state.window = Some(window);
    });
    with_visual_cx(cx, |_visual_cx, _entity| Ok(()))?;
    Ok(())
}
