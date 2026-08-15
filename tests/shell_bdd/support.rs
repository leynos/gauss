//! Shared durable-handle support for shell-level GPUI BDD scenarios.

use gauss::ui::Phase0Shell;
use gpui::{Context, Entity, TestAppContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

use crate::{
    durable_shell::DurableShell,
    lifecycle::{ensure_initial_draw, init_test_app},
};

#[derive(Default)]
pub(super) struct ScenarioState {
    shell: Option<DurableShell>,
}

crate::scenario_state!(ScenarioState; pub(super));

/// Create and draw a fresh shell, retaining only durable handles between steps.
pub fn fresh_shell_with(
    cx: &mut TestAppContext,
    create: impl FnOnce(&mut Context<Phase0Shell>) -> Phase0Shell,
) -> TestSupportResult<()> {
    reset_state();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| create(view_cx));
    ensure_initial_draw(visual_cx);
    with_state(|state| state.shell = Some(DurableShell::new(entity, visual_cx)));
    with_shell(cx, |_visual_cx, _view| Ok(()))
}

/// Rebuild a visual context for the current scenario's durable handles.
pub fn with_shell<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
) -> TestSupportResult<R> {
    let stored_shell = with_state(|state| state.shell.clone());
    let Some(scenario_shell) = stored_shell else {
        return Err(TestSupportError::missing(
            "scenario handles",
            "fresh shell must be created before using it",
        ));
    };
    scenario_shell.with_visual_cx(cx, f)
}

/// Read the shell quit-request state through durable scenario handles.
///
/// Scenario binaries invoke this expansion only where their Gherkin bindings
/// assert quit behaviour, preserving unused-helper detection elsewhere.
#[macro_export]
macro_rules! shell_did_request_quit {
    ($cx:expr) => {
        $crate::support::with_shell($cx, |visual_cx, view| {
            Ok(visual_cx.read(|app| view.read(app).did_request_quit()))
        })
    };
}
