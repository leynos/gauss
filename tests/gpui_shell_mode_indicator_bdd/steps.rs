//! Step definitions and scenario-local state for the Phase 0 shell mode
//! indicator BDD scenario.
//!
//! This is the first GPUI behavioural test migrated onto the `rstest-bdd`
//! 0.6.0 GPUI harness (`rstest_bdd_harness_gpui::GpuiHarness`). GPUI scenarios
//! share durable handles (`gpui::Entity` plus `gpui::AnyWindowHandle`) across
//! steps through a thread-local `RefCell`, rebuilding a `VisualTestContext`
//! per step from the harness-provided `gpui::TestAppContext`. The pattern, the
//! two-sided reset protocol, and the `#[serial]` requirement are documented in
//! `docs/rstest-bdd-users-guide.md` ("Stateful GPUI scenarios with durable
//! handles") and `docs/rstest-bdd-v0-6-0-migration-guide.md`.
//!
//! Note: the user's-guide playbook is written against a newer `gpui` than this
//! crate's `0.2.2`. The step shapes below use the `0.2.2` API: a two-argument
//! `add_window_view` closure, `gpui::Window::window_handle` to capture the
//! durable handle, and `VisualTestContext::from_window` returning the context
//! by value (not an `Option`).

use std::cell::RefCell;

use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, AppContext, Entity, TestAppContext, VisualTestContext};
use rstest::fixture;
use rstest_bdd_macros::{given, then, when};

/// Durable, copyable handles to the shell view and its window, shared across
/// the steps of a single serial scenario.
#[derive(Default)]
struct ScenarioState {
    entity: Option<Entity<Phase0Shell>>,
    window: Option<AnyWindowHandle>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

/// Clear the thread-local scenario state. Run after every scenario via the
/// `Drop` guard so a reused serial test thread starts clean.
fn reset_state_after_scenario() {
    SCENARIO_STATE.with(|state| *state.borrow_mut() = ScenarioState::default());
}

/// Clear the thread-local scenario state before storing fresh handles. Covers
/// the case where a previous scenario aborted in a way that bypassed `Drop`.
fn reset_state_before_assignment() {
    reset_state_after_scenario();
}

fn store_handles(entity: Entity<Phase0Shell>, window: AnyWindowHandle) {
    SCENARIO_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.entity = Some(entity);
        state.window = Some(window);
    });
}

fn current_handles() -> (Entity<Phase0Shell>, AnyWindowHandle) {
    SCENARIO_STATE.with(|cell| {
        let state = cell.borrow();
        let Some(entity) = state.entity.clone() else {
            panic!("scenario state is missing the shell entity; the opening step must run first")
        };
        let Some(window) = state.window else {
            panic!("scenario state is missing the window handle; the opening step must run first")
        };
        (entity, window)
    })
}

/// Drop-based guard that resets thread-local scenario state on every unwind
/// path (success, assertion failure, and panic alike).
pub(crate) struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state_after_scenario();
    }
}

/// Fixture that resets scenario state before the scenario runs and returns a
/// guard that resets it again on teardown (the two-sided reset protocol).
#[fixture]
pub(crate) fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

#[given("a Phase 0 shell window is open")]
fn shell_window_is_open(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    reset_state_before_assignment();
    // Register actions and keybindings so keystroke dispatch (for example the
    // `tab` edge-mode toggle) resolves in the headless app.
    cx.update(gauss::ui::init);

    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    // Draw once so the shell initialises, then capture the durable window
    // handle. gpui 0.2.2 exposes the handle via `Window`, not
    // `VisualTestContext`.
    let window = visual_cx.update(|window, app| {
        let _draw = window.draw(app);
        window.window_handle()
    });
    visual_cx.run_until_parked();

    store_handles(entity, window);
}

#[when("the user presses {key}")]
fn user_presses_key(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext, key: String) {
    let key_name = key.trim_matches('"').to_owned();
    let (_entity, window) = current_handles();
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    visual_cx.simulate_keystrokes(&key_name);
    visual_cx.run_until_parked();
}

#[when("the shell switches to manipulate mode")]
fn shell_switches_to_manipulate_mode(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    let (entity, window) = current_handles();
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    visual_cx.update_entity(&entity, |shell, view_cx| {
        shell.enter_manipulate_mode_for_tests();
        view_cx.notify();
    });
    visual_cx.run_until_parked();
}

#[then("the mode indicator reads {expected}")]
fn mode_indicator_reads(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    expected: String,
) {
    let expected_line = expected.trim_matches('"').to_owned();
    let (entity, _window) = current_handles();
    let actual = cx.read_entity(&entity, |shell, _app| shell.mode_status_line_for_tests());
    assert_eq!(actual, expected_line);
}
