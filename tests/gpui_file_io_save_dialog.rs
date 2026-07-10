//! Behavioural coverage for GPUI Save and web-ready export file dialogs.

mod common;
#[path = "gpui_file_io_save_dialog/fixtures.rs"]
mod fixtures;

use std::{cell::RefCell, path::Path};

use common::{
    ensure_initial_draw,
    file_io::{DurableShell, TempSvgFile},
    init_test_app,
};
use gauss::svg::metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX};
use gauss::ui::{ExportSvgWebReady, Phase0Shell, SaveSvg};
use gpui::TestAppContext;
use rstest::fixture;
use rstest_bdd_macros::{scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

#[derive(Clone, Copy)]
pub(crate) enum ExportAction {
    Save,
    WebReady,
}

#[derive(Default)]
struct ScenarioState {
    shell: Option<DurableShell>,
    temp_svg: Option<TempSvgFile>,
    action: Option<ExportAction>,
}

thread_local! {
    static STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

fn with_state<R>(f: impl FnOnce(&mut ScenarioState) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
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

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone())
        .ok_or_else(|| TestSupportError::missing("shell handles", "set by the Given step"))
}

pub(crate) fn prepare_shell(
    cx: &mut TestAppContext,
    action: ExportAction,
    configure: impl FnOnce(&mut Phase0Shell),
) -> Result<(), TestSupportError> {
    reset_state();
    init_test_app(cx);
    let (shell_entity, initial_visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(initial_visual_cx);
    let shell_handles = DurableShell::new(shell_entity, initial_visual_cx);
    shell_handles.with_visual_cx(cx, |step_visual_cx, entity_ref| {
        entity_ref.update(step_visual_cx, |shell_instance, _cx| {
            configure(shell_instance);
        });
        Ok(())
    })?;
    with_state(|state| {
        state.shell = Some(shell_handles);
        state.action = Some(action);
    });
    Ok(())
}

#[then("no file path prompt is visible")]
fn no_file_path_prompt(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(
            "expected no file path prompt before export",
        ));
    }
    Ok(())
}

#[when("the configured export action is requested")]
fn request_export(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let action = with_state(|state| state.action).ok_or_else(|| {
        TestSupportError::missing("configured export action", "set by the Given step")
    })?;
    shell()?.with_visual_cx(cx, |visual_cx, _entity| {
        match action {
            ExportAction::Save => visual_cx.dispatch_action(SaveSvg),
            ExportAction::WebReady => visual_cx.dispatch_action(ExportSvgWebReady),
        }
        visual_cx.run_until_parked();
        Ok(())
    })?;
    cx.run_until_parked();
    Ok(())
}

#[then("a file path prompt is visible")]
fn file_path_prompt(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if !cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(
            "expected the export action to display a file path prompt",
        ));
    }
    Ok(())
}

#[when("a temporary save path is selected")]
fn select_temporary_save_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let temp_svg = TempSvgFile::create("gauss-test-save-dialog")?;
    let path = temp_svg.path().as_std_path().to_path_buf();
    cx.simulate_new_path_selection(|_directory: &Path| Some(path.clone()));
    cx.run_until_parked();
    with_state(|state| state.temp_svg = Some(temp_svg));
    Ok(())
}

fn save_outcome(cx: &TestAppContext) -> Option<(Option<std::path::PathBuf>, Option<String>)> {
    cx.read(|app| {
        shell().ok().map(|handles| {
            let shell = handles.entity().read(app);
            (
                shell.last_saved_path().map(Path::to_path_buf),
                shell.last_save_error().map(str::to_owned),
            )
        })
    })
}

#[then("the selected save path is recorded")]
fn selected_save_path_recorded(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| {
        state
            .temp_svg
            .as_ref()
            .map(|temp| temp.path().as_std_path().to_path_buf())
    });
    let actual = save_outcome(cx).and_then(|(path, _error)| path);
    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected saved path {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

fn saved_contents() -> Result<String, TestSupportError> {
    with_state(|state| state.temp_svg.as_ref().map(TempSvgFile::read_to_string))
        .ok_or_else(|| TestSupportError::missing("temporary SVG", "set by the When step"))?
}

fn require_contents(fragment: &str, context: &str) -> Result<(), TestSupportError> {
    let contents = saved_contents()?;
    if !contents.contains(fragment) {
        return Err(TestSupportError::expectation(format!(
            "{context}; saved SVG was: {contents}"
        )));
    }
    Ok(())
}

#[then("the saved SVG contains the demo shape")]
fn saved_svg_contains_demo_shape() -> Result<(), TestSupportError> {
    require_contents(
        r#"<path d="M 10 10 L 90 10 L 90 90 L 10 90 Z""#,
        "saved SVG did not contain the demo shape",
    )
}

#[then("the saved SVG contains the canonical Gauss metadata namespace")]
fn saved_svg_contains_namespace() -> Result<(), TestSupportError> {
    require_contents(
        &format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#),
        "saved SVG did not contain the canonical Gauss metadata namespace",
    )
}

#[then("no save path is recorded")]
fn no_save_path_recorded(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if save_outcome(cx).and_then(|(path, _error)| path).is_some() {
        return Err(TestSupportError::expectation(
            "export validation failure recorded a save path",
        ));
    }
    Ok(())
}

fn require_save_error(cx: &TestAppContext, expected: &str) -> Result<(), TestSupportError> {
    let error = save_outcome(cx).and_then(|(_path, error)| error);
    if !error.as_deref().is_some_and(|text| text.contains(expected)) {
        return Err(TestSupportError::expectation(format!(
            "expected save error containing {expected:?}, found {error:?}"
        )));
    }
    Ok(())
}

#[then("the save error reports the missing gradient resource")]
fn save_error_reports_gradient(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_save_error(cx, "missing gradient resource")
}

#[then("the save error reports the missing pattern resource")]
fn save_error_reports_pattern(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_save_error(cx, "missing pattern resource")
}

#[then("no SVG file is written")]
fn no_svg_file_written() -> Result<(), TestSupportError> {
    let exists = with_state(|state| state.temp_svg.as_ref().is_some_and(TempSvgFile::exists));
    if exists {
        return Err(TestSupportError::expectation(
            "export validation failure wrote an SVG file",
        ));
    }
    Ok(())
}

#[then("no save error is reported")]
fn no_save_error(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if let Some(error) = save_outcome(cx).and_then(|(_path, error)| error) {
        return Err(TestSupportError::expectation(format!(
            "expected web-ready export to succeed, found {error}"
        )));
    }
    Ok(())
}

#[then("the saved SVG is web-ready and contains no Gauss metadata")]
fn saved_svg_is_web_ready() -> Result<(), TestSupportError> {
    let contents = saved_contents()?;
    let has_demo_shape = contents.contains(r#"<path d="M 10 10 L 90 10 L 90 90 L 10 90 Z""#);
    let has_gauss_metadata = contents.contains(GAUSS_METADATA_NAMESPACE)
        || contents.contains(&format!("{GAUSS_METADATA_PREFIX}:"))
        || contents.contains("<metadata>");
    if !has_demo_shape || has_gauss_metadata {
        return Err(TestSupportError::expectation(format!(
            "expected web-ready SVG without Gauss metadata: {contents}"
        )));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/file_io_save_dialog.feature",
    name = "Save prompts for a path and writes SVG",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn save_writes_svg(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_save_dialog.feature",
    name = "Save reports a dangling gradient reference",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn save_dangling_gradient_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {
}

#[scenario(
    path = "tests/features/file_io_save_dialog.feature",
    name = "Save reports a dangling pattern reference",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn save_dangling_pattern_scenario(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_save_dialog.feature",
    name = "Web-ready export strips Gauss metadata",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn web_ready_strips_metadata(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_save_dialog.feature",
    name = "Web-ready export reports a dangling gradient reference",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn web_ready_dangling_gradient_scenario(
    #[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(
    path = "tests/features/file_io_save_dialog.feature",
    name = "Web-ready export reports a dangling pattern reference",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn web_ready_dangling_pattern_scenario(
    #[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
