//! Shared durable GPUI scenario state for tooling behavioural tests.

use std::{any::Any, cell::RefCell};

use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualContext, VisualTestContext};
use rstest::fixture;
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct ScenarioState {
    entity: Option<Entity<Phase0Shell>>,
    window: Option<AnyWindowHandle>,
    data: Option<Box<dyn Any>>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

fn reset_state_after_scenario() {
    SCENARIO_STATE.with(|cell| *cell.borrow_mut() = ScenarioState::default());
}

fn reset_state_before_assignment() {
    // A serial test thread can be reused after an earlier scenario failed.
    reset_state_after_scenario();
}

/// Clears thread-local scenario state on every exit path.
pub struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state_after_scenario();
    }
}

/// Provides two-sided cleanup for a stateful GPUI scenario.
#[fixture]
pub fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

/// Opens a fresh shell and stores its durable handles and scenario-specific data.
pub fn initialize<T: 'static>(
    cx: &mut TestAppContext,
    setup: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<T>,
) -> TestSupportResult<()> {
    reset_state_before_assignment();
    crate::common::init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    crate::common::ensure_initial_draw(visual_cx);
    let data = setup(visual_cx, &entity)?;
    let window = visual_cx.window_handle();
    SCENARIO_STATE.with(|cell| {
        *cell.borrow_mut() = ScenarioState {
            entity: Some(entity),
            window: Some(window),
            data: Some(Box::new(data)),
        };
    });
    Ok(())
}

/// Removes and downcasts the scenario's typed scratch data for one step.
fn take_scenario_data<T: 'static>() -> TestSupportResult<Box<T>> {
    let missing_data = || {
        TestSupportError::missing(
            std::any::type_name::<T>(),
            "scenario-specific data assigned by the Given step",
        )
    };
    let stored_data = SCENARIO_STATE.with(|cell| cell.borrow_mut().data.take());
    let Some(scenario_data) = stored_data else {
        return Err(missing_data());
    };
    match scenario_data.downcast::<T>() {
        Ok(typed_data) => Ok(typed_data),
        Err(original_data) => {
            SCENARIO_STATE.with(|cell| cell.borrow_mut().data = Some(original_data));
            Err(missing_data())
        }
    }
}

/// Rebuilds the visual context and exposes the scenario's typed scratch data.
pub fn with_visual_cx<T: 'static, R>(
    cx: &mut TestAppContext,
    operation: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>, &mut T) -> TestSupportResult<R>,
) -> TestSupportResult<R> {
    let handles = SCENARIO_STATE.with(|cell| {
        let state = cell.borrow();
        (state.entity.clone(), state.window)
    });
    let (Some(entity), Some(window)) = handles else {
        return Err(TestSupportError::missing(
            "scenario handles",
            "assigned by the scenario's Given step",
        ));
    };
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    let mut typed_data = take_scenario_data()?;
    let result = operation(&mut visual_cx, &entity, &mut typed_data);
    SCENARIO_STATE.with(|cell| cell.borrow_mut().data = Some(typed_data));
    result
}
