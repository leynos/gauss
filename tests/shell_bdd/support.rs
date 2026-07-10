//! Shared durable-handle support for shell-level GPUI BDD scenarios.

use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Context, Entity, TestAppContext, VisualContext, VisualTestContext};
use rstest::fixture;
use std::cell::RefCell;
use test_support::{TestSupportError, TestSupportResult};

use crate::common::{ensure_initial_draw, init_test_app};

#[derive(Default)]
struct ScenarioState {
    entity: Option<Entity<Phase0Shell>>,
    window: Option<AnyWindowHandle>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

fn reset_state() {
    SCENARIO_STATE.with(|cell| *cell.borrow_mut() = ScenarioState::default());
}

/// Guard that clears durable GPUI handles on every scenario exit path.
pub struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state();
    }
}

#[fixture]
pub fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state();
    ScenarioStateCleanup
}

/// Create and draw a fresh shell, retaining only durable handles between steps.
pub fn fresh_shell_with(
    cx: &mut TestAppContext,
    create: impl FnOnce(&mut Context<Phase0Shell>) -> Phase0Shell,
) {
    reset_state();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| create(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    SCENARIO_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.entity = Some(entity);
        state.window = Some(window);
    });
}

/// Rebuild a visual context for the current scenario's durable handles.
pub fn with_shell<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
) -> TestSupportResult<R> {
    let handles = SCENARIO_STATE.with(|cell| {
        let state = cell.borrow();
        (state.entity.clone(), state.window)
    });
    let (Some(entity), Some(window)) = handles else {
        return Err(TestSupportError::missing(
            "scenario handles",
            "fresh shell must be created before using it",
        ));
    };
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    f(&mut visual_cx, &entity)
}
