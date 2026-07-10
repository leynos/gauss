//! Behavioural coverage for loading SVG documents through the GPUI Open dialog.

mod common;
#[path = "gpui_file_io_open_dialog/fixtures.rs"]
mod fixtures;

use std::{cell::RefCell, path::Path};

use common::{
    ensure_initial_draw,
    file_io::{DurableShell, TempSvgFile},
    init_test_app,
};
use gauss::model::Paint;
use gauss::svg::metadata::GAUSS_METADATA_PREFIX;
use gauss::ui::{OpenSvg, Phase0Shell};
use gpui::TestAppContext;
use rstest::fixture;
use rstest_bdd_macros::{scenario, then, when};
use serial_test::serial;
use test_support::TestSupportError;

#[derive(Default)]
struct ScenarioState {
    shell: Option<DurableShell>,
    temp_svg: Option<TempSvgFile>,
    initial_resources: Option<(usize, usize, usize)>,
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

pub(crate) fn prepare_svg(
    cx: &mut TestAppContext,
    prefix: &str,
    contents: &str,
) -> Result<(), TestSupportError> {
    reset_state();
    init_test_app(cx);
    let temp_svg = TempSvgFile::create(prefix)?;
    temp_svg.write(contents)?;
    let (entity, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);
    let shell = DurableShell::new(entity, visual_cx);
    let initial_resources = cx.read(|app| {
        let resources = shell.entity().read(app).resources();
        (
            resources.gradient_count(),
            resources.pattern_count(),
            resources.symbol_count(),
        )
    });
    with_state(|state| {
        state.shell = Some(shell);
        state.temp_svg = Some(temp_svg);
        state.initial_resources = Some(initial_resources);
    });
    Ok(())
}

#[then("no file path prompt is visible")]
fn no_file_path_prompt(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(
            "expected no file path prompt before Open",
        ));
    }
    Ok(())
}

#[when("Open is requested")]
fn request_open(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual_cx(cx, |visual_cx, _entity| {
        visual_cx.dispatch_action(OpenSvg);
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
            "expected Open to display a file path prompt",
        ));
    }
    Ok(())
}

#[when("the temporary SVG is selected")]
fn select_temporary_svg(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let path = with_state(|state| {
        state
            .temp_svg
            .as_ref()
            .map(|temp| temp.path().as_std_path().to_path_buf())
    })
    .ok_or_else(|| TestSupportError::missing("temporary SVG", "set by the Given step"))?;
    cx.simulate_new_path_selection(|_directory: &Path| Some(path.clone()));
    cx.run_until_parked();
    Ok(())
}

#[then("the selected SVG path is recorded")]
fn selected_path_recorded(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| {
        state
            .temp_svg
            .as_ref()
            .map(|temp| temp.path().as_std_path().to_path_buf())
    });
    let actual = cx.read(|app| {
        shell().ok().and_then(|handles| {
            handles
                .entity()
                .read(app)
                .last_opened_path()
                .map(Path::to_path_buf)
        })
    });
    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected opened path {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

fn document_shape_count(cx: &TestAppContext) -> Option<usize> {
    cx.read(|app| {
        shell()
            .ok()
            .map(|handles| handles.entity().read(app).document().len())
    })
}

#[then("the document contains one shape")]
fn document_contains_one_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if document_shape_count(cx) != Some(1) {
        return Err(TestSupportError::expectation(format!(
            "expected one shape, found {:?}",
            document_shape_count(cx)
        )));
    }
    Ok(())
}

#[then("the document has no gradient or pattern resources")]
fn document_has_no_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let counts = cx.read(|app| {
        shell().ok().map(|handles| {
            let resources = handles.entity().read(app).resources();
            (resources.gradient_count(), resources.pattern_count())
        })
    });
    if counts != Some((0, 0)) {
        return Err(TestSupportError::expectation(format!(
            "expected no gradient or pattern resources, found {counts:?}"
        )));
    }
    Ok(())
}

#[then("the document contains one gradient and one pattern")]
fn document_contains_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let counts = cx.read(|app| {
        shell().ok().map(|handles| {
            let resources = handles.entity().read(app).resources();
            (resources.gradient_count(), resources.pattern_count())
        })
    });
    if counts != Some((1, 1)) {
        return Err(TestSupportError::expectation(format!(
            "expected one gradient and one pattern, found {counts:?}"
        )));
    }
    Ok(())
}

#[then("the imported shape references the gradient and pattern")]
fn imported_shape_references_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let matches = cx.read(|app| {
        shell().ok().and_then(|handles| {
            let shell = handles.entity().read(app);
            let gradient = shell.resources().gradient_id_for_svg_id("sunset")?;
            let pattern = shell.resources().pattern_id_for_svg_id("dots")?;
            let shape = shell.document().shape_at(0)?;
            Some(
                shape.style.stroke == Paint::gradient(gradient)
                    && shape.style.fill == Paint::pattern(pattern),
            )
        })
    });
    if matches != Some(true) {
        return Err(TestSupportError::expectation(
            "imported shape did not reference the imported gradient and pattern",
        ));
    }
    Ok(())
}

#[then("the original document and resources are preserved")]
fn original_state_preserved(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected_resources = with_state(|state| state.initial_resources);
    let actual = cx.read(|app| {
        shell().ok().map(|handles| {
            let shell = handles.entity().read(app);
            let resources = shell.resources();
            (
                shell.document().len(),
                resources.gradient_count(),
                resources.pattern_count(),
                resources.symbol_count(),
            )
        })
    });
    let expected = expected_resources.map(|(g, p, s)| (1, g, p, s));
    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected original document and resources {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

fn open_error(cx: &TestAppContext) -> Option<String> {
    cx.read(|app| {
        shell().ok().and_then(|handles| {
            handles
                .entity()
                .read(app)
                .last_open_error()
                .map(str::to_owned)
        })
    })
}

#[then("the open error reports a missing resource")]
fn open_error_reports_missing_resource(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let error = open_error(cx);
    if !error
        .as_deref()
        .is_some_and(|text| text.contains("missing resource"))
    {
        return Err(TestSupportError::expectation(format!(
            "expected a missing-resource error, found {error:?}"
        )));
    }
    Ok(())
}

#[then("no open error is reported")]
fn no_open_error(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    if let Some(error) = open_error(cx) {
        return Err(TestSupportError::expectation(format!(
            "expected Open to succeed, found {error}"
        )));
    }
    Ok(())
}

#[then("the open error reports the canonical Gauss namespace declaration")]
fn open_error_reports_namespace(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = format!("xmlns:{GAUSS_METADATA_PREFIX}");
    let error = open_error(cx);
    if !error
        .as_deref()
        .is_some_and(|text| text.contains(&expected))
    {
        return Err(TestSupportError::expectation(format!(
            "expected an error mentioning {expected}, found {error:?}"
        )));
    }
    Ok(())
}
#[scenario(
    path = "tests/features/file_io_open_dialog.feature",
    name = "Open loads the selected SVG",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_selected_svg(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_open_dialog.feature",
    name = "Open loads resource definitions and paint references",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_resources(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_open_dialog.feature",
    name = "Open reports a missing resource reference",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_missing_resource(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_open_dialog.feature",
    name = "Open accepts the canonical Gauss metadata namespace",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_canonical_metadata(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_open_dialog.feature",
    name = "Open rejects a non-canonical Gauss metadata prefix",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_noncanonical_prefix(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
