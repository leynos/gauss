//! Behavioural coverage for loading SVG documents through the GPUI Open dialog.

mod common;
#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "gpui_file_io_open_dialog/fixtures.rs"]
mod fixtures;
#[path = "common/path_prompt.rs"]
mod path_prompt;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "common/temp_svg.rs"]
mod temp_svg;
#[path = "common/temp_svg_write.rs"]
mod temp_svg_write;

use std::path::Path;

use common::{ensure_initial_draw, init_test_app};
use durable_shell::DurableShell;
use gauss::model::Paint;
use gauss::svg::metadata::GAUSS_METADATA_PREFIX;
use gauss::ui::{OpenSvg, Phase0Shell};
use gpui::TestAppContext;
use path_prompt::{assert_no_path_prompt, assert_path_prompt};
use rstest_bdd_macros::{scenario, then, when};
use serial_test::serial;
use temp_svg::TempSvgFile;
use test_support::TestSupportError;

#[derive(Default)]
struct ScenarioState {
    shell: Option<DurableShell>,
    temp_svg: Option<TempSvgFile>,
    initial_resources: Option<(usize, usize, usize)>,
}

crate::scenario_state!(ScenarioState);

/// Clone the durable shell handle out of thread-local scenario state.
///
/// # Errors
///
/// Returns `Err` if the Given step that populates the handle has not run yet.
fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone())
        .ok_or_else(|| TestSupportError::missing("shell handles", "set by the Given step"))
}

/// Arrange a fresh Phase 0 shell and a temporary SVG file with `contents`,
/// resetting scenario state first and recording the shell's initial resource
/// counts for later comparison.
///
/// # Errors
///
/// Returns `Err` if the temporary SVG cannot be created or written.
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
        let resources = shell.entity.read(app).resources();
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
    assert_no_path_prompt(cx, "expected no file path prompt before Open")
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
    assert_path_prompt(cx, "expected Open to display a file path prompt")
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
    let handles = shell()?;
    let actual = cx.read(|app| {
        handles
            .entity
            .read(app)
            .last_opened_path()
            .map(Path::to_path_buf)
    });
    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected opened path {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

/// Read the number of shapes in the shell's document.
///
/// # Errors
///
/// Returns `Err` if the Given step that creates the shell has not run.
fn document_shape_count(cx: &TestAppContext) -> Result<usize, TestSupportError> {
    let handles = shell()?;
    Ok(cx.read(|app| handles.entity.read(app).document().len()))
}

#[then("the document contains one shape")]
fn document_contains_one_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let count = document_shape_count(cx)?;
    if count != 1 {
        return Err(TestSupportError::expectation(format!(
            "expected one shape, found {count}"
        )));
    }
    Ok(())
}

/// Assert the imported document holds `expected` gradient and pattern counts.
///
/// `context` describes the expectation so each step keeps its own wording.
fn require_resource_counts(
    cx: &TestAppContext,
    expected: (usize, usize),
    context: &str,
) -> Result<(), TestSupportError> {
    let handles = shell()?;
    let counts = cx.read(|app| {
        let resources = handles.entity.read(app).resources();
        (resources.gradient_count(), resources.pattern_count())
    });
    if counts != expected {
        return Err(TestSupportError::expectation(format!(
            "{context}, found {counts:?}"
        )));
    }
    Ok(())
}

#[then("the document has no gradient or pattern resources")]
fn document_has_no_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_resource_counts(cx, (0, 0), "expected no gradient or pattern resources")
}

#[then("the document contains one gradient and one pattern")]
fn document_contains_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    require_resource_counts(cx, (1, 1), "expected one gradient and one pattern")
}

#[then("the imported shape references the gradient and pattern")]
fn imported_shape_references_resources(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let handles = shell()?;
    let matches = cx.read(|app| {
        let shell = handles.entity.read(app);
        let gradient = shell.resources().gradient_id_for_svg_id("sunset")?;
        let pattern = shell.resources().pattern_id_for_svg_id("dots")?;
        let shape = shell.document().shape_at(0)?;
        Some(
            shape.style.stroke == Paint::gradient(gradient)
                && shape.style.fill == Paint::pattern(pattern),
        )
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
    let handles = shell()?;
    let actual = cx.read(|app| {
        let shell = handles.entity.read(app);
        let resources = shell.resources();
        (
            shell.document().len(),
            resources.gradient_count(),
            resources.pattern_count(),
            resources.symbol_count(),
        )
    });
    let expected = expected_resources
        .map(|(g, p, s)| (1, g, p, s))
        .ok_or_else(|| {
            TestSupportError::missing("initial resource counts", "set by the Given step")
        })?;
    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "expected original document and resources {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

/// Read the shell's last recorded Open error.
///
/// The shell handle is resolved before reading, so absent scenario state is an
/// error rather than an absent error message. That keeps `no open error is
/// reported` from passing when no shell was ever created.
///
/// # Errors
///
/// Returns `Err` if the Given step that creates the shell has not run.
fn open_error(cx: &TestAppContext) -> Result<Option<String>, TestSupportError> {
    let handles = shell()?;
    Ok(cx.read(|app| {
        handles
            .entity
            .read(app)
            .last_open_error()
            .map(str::to_owned)
    }))
}

#[then("the open error reports a missing resource")]
fn open_error_reports_missing_resource(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let error = open_error(cx)?;
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
    if let Some(error) = open_error(cx)? {
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
    let error = open_error(cx)?;
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
