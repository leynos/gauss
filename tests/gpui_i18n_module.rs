//! GPUI behavioural scenarios for localized Phase 0 shell status text.

mod common;

use std::{cell::RefCell, collections::HashMap};

use common::{init_test_app, make_test_catalog, test_french_catalog};
use gauss::{
    i18n::{Catalog, Locale, Localizer},
    model::ToolMode,
    ui::phase0_shell::Phase0Shell,
};
use gpui::{Entity, TestAppContext};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

#[derive(Default)]
struct ScenarioState {
    shell: Option<Entity<Phase0Shell>>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

fn with_state<R>(f: impl FnOnce(&mut ScenarioState) -> R) -> R {
    SCENARIO_STATE.with(|cell| f(&mut cell.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = ScenarioState::default());
}

struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state();
    }
}

#[fixture]
fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state();
    ScenarioStateCleanup
}

/// Builds the bilingual localizer shape shared by these scenarios: a French and
/// an English catalogue with en-GB as the default (and therefore fallback)
/// locale. Keeping the default locale in one place avoids repeating it per
/// helper.
fn build_localizer(french: Catalog, english: Catalog) -> Localizer {
    let catalogs = HashMap::from([(Locale::fr_fr(), french), (Locale::en_gb(), english)]);
    Localizer::with_catalogs(catalogs, Locale::en_gb())
}

fn setup_localizer() -> Localizer {
    build_localizer(test_french_catalog(), Catalog::default_en_gb())
}

fn partial_french_localizer() -> Localizer {
    // The French catalogue is deliberately incomplete: only `tool_mode.draw` is
    // present, so every other key must fall back to en-GB. The en-GB catalogue
    // uses a distinctive `edge_mode.line` marker so the fallback path is
    // observable in the mode status.
    let partial_fr_catalog = make_test_catalog(&[("tool_mode.draw", "Dessiner")]);
    let en_catalog = make_test_catalog(&[
        ("tool_mode.draw", "Draw"),
        ("tool_mode.manipulate", "Manipulate"),
        ("edge_mode.line", "Line via en-GB catalogue"),
        ("edge_mode.bezier_auto", "Bezier (auto)"),
        ("tool.status.mode_with_edge", "Mode: {tool} ({edge})"),
        ("tool.status.mode", "Mode: {tool}"),
    ]);
    build_localizer(partial_fr_catalog, en_catalog)
}

fn assign_shell(cx: &mut TestAppContext, localizer: Localizer, locale: Locale) {
    reset_state();
    init_test_app(cx);
    let (shell, _visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell_instance = Phase0Shell::new(view_cx);
        shell_instance.set_localizer(localizer);
        shell_instance.set_locale(locale);
        shell_instance
    });
    with_state(|state| state.shell = Some(shell));
}

fn current_shell() -> Result<Entity<Phase0Shell>, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| {
        TestSupportError::missing(
            "Phase 0 shell",
            "a Given step must assign the shell before it is used",
        )
    })
}

fn status_line(cx: &TestAppContext) -> Result<String, TestSupportError> {
    let shell_entity = current_shell()?;
    Ok(shell_entity.read_with(cx, |shell, _cx| shell.mode_status_line_for_tests()))
}

#[given("a fresh Phase 0 shell using the default English catalogue")]
fn default_english_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assign_shell(cx, Localizer::new(), Locale::en_gb());
    current_shell()?;
    Ok(())
}

#[given("a fresh Phase 0 shell using the test French catalogue")]
fn test_french_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assign_shell(cx, setup_localizer(), Locale::fr_fr());
    current_shell()?;
    Ok(())
}

#[given("a fresh Phase 0 shell with English and French catalogues")]
fn bilingual_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assign_shell(cx, setup_localizer(), Locale::en_gb());
    current_shell()?;
    Ok(())
}

#[given("a fresh Phase 0 shell with a partial French catalogue")]
fn partial_french_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assign_shell(cx, partial_french_localizer(), Locale::fr_fr());
    current_shell()?;
    Ok(())
}

#[when("the shell locale is switched to French")]
fn switch_locale_to_french(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    current_shell()?.update(cx, |shell, _cx| shell.set_locale(Locale::fr_fr()));
    Ok(())
}

#[when("the shell tool mode is changed to manipulate")]
fn switch_to_manipulate(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    current_shell()?.update(cx, |shell, _cx| {
        shell.set_tool_mode_for_tests(ToolMode::Manipulate);
    });
    Ok(())
}

fn status_contains(
    cx: &TestAppContext,
    expected: &str,
    description: &str,
) -> Result<(), TestSupportError> {
    let status = status_line(cx)?;
    if status.contains(expected) {
        return Ok(());
    }
    Err(TestSupportError::expectation(format!(
        "expected {description} '{expected}' in mode status, got: {status}"
    )))
}

#[then("the mode status shows the tool label {label}")]
fn mode_status_shows_tool_label(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    label: String,
) -> Result<(), TestSupportError> {
    status_contains(cx, &label, "tool label")
}

#[then("the mode status shows the edge label {label}")]
fn mode_status_shows_edge_label(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    label: String,
) -> Result<(), TestSupportError> {
    status_contains(cx, &label, "edge label")
}

#[then("the mode status shows the fallback edge label")]
fn mode_status_shows_fallback_edge_label(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    status_contains(cx, "Line via en-GB catalogue", "fallback edge label")
}

#[then("the mode status omits the edge mode")]
fn mode_status_omits_edge_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let status = status_line(cx)?;
    // Manipulate mode has no edge; the active edge label ("Ligne", the Line
    // edge mode in the test French catalogue) must therefore be absent.
    let edge_label = "Ligne";
    if !status.contains(edge_label) {
        return Ok(());
    }
    Err(TestSupportError::expectation(format!(
        "expected manipulate mode status to omit the edge label '{edge_label}', got: {status}"
    )))
}

#[scenario(
    path = "tests/features/gpui_i18n_status.feature",
    name = "Default English catalogue localizes the mode status",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn default_english_catalogue(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/gpui_i18n_status.feature",
    name = "Injected French catalogue localizes the mode status",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn injected_french_catalogue(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/gpui_i18n_status.feature",
    name = "Switching locale updates the mode status",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn switching_locale(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/gpui_i18n_status.feature",
    name = "Missing French message falls back to the default locale",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn missing_french_message(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/gpui_i18n_status.feature",
    name = "Manipulate mode omits the edge mode from the status",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn manipulate_mode_status(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
