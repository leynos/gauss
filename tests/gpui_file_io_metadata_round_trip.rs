//! Behavioural coverage for preserving Gauss metadata through GPUI save and open flows.

#[path = "common/gpui_file_io_metadata_round_trip.rs"]
mod common;

#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "common/temp_svg.rs"]
mod temp_svg;
#[path = "common/temp_svg_read.rs"]
mod temp_svg_read;
#[path = "common/temp_svg_write.rs"]
mod temp_svg_write;

use std::path::Path;

use common::{ensure_initial_draw, init_test_app};
use durable_shell::DurableShell;
use gauss::model::ShapeId;
use gauss::svg::metadata::GAUSS_METADATA_NAMESPACE;
use gauss::ui::{OpenSvg, Phase0Shell, SaveSvg};
use gauss_core::test_helpers::shape_id_from_seed;
use gpui::TestAppContext;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use temp_svg::TempSvgFile;
use test_support::TestSupportError;

#[derive(Default)]
struct ScenarioState {
    shell: Option<DurableShell>,
    temp_svg: Option<TempSvgFile>,
    saved_contents: Option<String>,
    expected_id: Option<ShapeId>,
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

/// Which flavour of shell a scenario needs.
///
/// The save scenarios exercise the demo document that `Phase0Shell::new` seeds,
/// whereas the open scenarios start from the empty test shell so the imported
/// document is the only content present.
#[derive(Clone, Copy)]
enum ShellKind {
    Save,
    Open,
}

/// Build and register a Phase 0 shell window of the requested `kind`,
/// performing the initial draw before returning its durable handle.
fn create_shell(cx: &mut TestAppContext, kind: ShellKind) -> DurableShell {
    init_test_app(cx);
    let (entity, visual_cx) = match kind {
        ShellKind::Save => cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx)),
        ShellKind::Open => {
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx))
        }
    };
    ensure_initial_draw(visual_cx);
    DurableShell::new(entity, visual_cx)
}

/// Arrange a fresh demo-seeded shell for the save scenarios, resetting
/// scenario state first and letting `configure` adjust the shell before use.
///
/// # Errors
///
/// Returns `Err` if the visual context cannot be borrowed to run `configure`.
fn prepare_save_shell(
    cx: &mut TestAppContext,
    configure: impl FnOnce(&mut Phase0Shell),
) -> Result<(), TestSupportError> {
    reset_state();
    let shell_handles = create_shell(cx, ShellKind::Save);
    shell_handles.with_visual_cx(cx, |visual_cx, entity| {
        entity.update(visual_cx, |shell, _cx| configure(shell));
        Ok(())
    })?;
    with_state(|state| state.shell = Some(shell_handles));
    Ok(())
}

/// Arrange an empty test shell and a temporary SVG file with `contents` for
/// the open scenarios, resetting scenario state first.
///
/// # Errors
///
/// Returns `Err` if the temporary SVG cannot be created or written.
fn prepare_open_svg(
    cx: &mut TestAppContext,
    prefix: &str,
    contents: &str,
) -> Result<(), TestSupportError> {
    reset_state();
    let temp_svg = TempSvgFile::create(prefix)?;
    temp_svg.write(contents)?;
    let shell = create_shell(cx, ShellKind::Open);
    with_state(|state| {
        state.shell = Some(shell);
        state.temp_svg = Some(temp_svg);
    });
    Ok(())
}

#[given("a shell whose demo shape has a known Gauss identifier")]
fn shell_with_known_id(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected_id = shape_id_from_seed(42_u128);
    prepare_save_shell(cx, |shell| {
        if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
            shape.id = expected_id;
        }
    })?;
    with_state(|state| state.expected_id = Some(expected_id));
    Ok(())
}

#[given("a shell with a Gauss project metadata block")]
fn shell_with_metadata(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_save_shell(cx, |shell| {
        shell.set_gauss_metadata_block_for_tests(Some(
            "\n<gauss:project>test-project</gauss:project>".to_owned(),
        ));
    })
}

#[when("the document is saved to a temporary SVG")]
fn save_to_temporary_svg(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual_cx(cx, |visual_cx, _entity| {
        visual_cx.dispatch_action(SaveSvg);
        visual_cx.run_until_parked();
        Ok(())
    })?;
    cx.run_until_parked();
    let temp_svg = TempSvgFile::create("gauss-test-metadata-save")?;
    let path = temp_svg.path().as_std_path().to_path_buf();
    cx.simulate_new_path_selection(|_directory: &Path| Some(path.clone()));
    cx.run_until_parked();
    let handles = shell()?;
    let saved = cx.read(|app| {
        handles
            .entity
            .read(app)
            .last_saved_path()
            .map(Path::to_path_buf)
    });
    if saved.as_deref() != Some(path.as_path()) {
        return Err(TestSupportError::expectation(format!(
            "expected saved path {}, found {saved:?}",
            path.display()
        )));
    }
    let contents = temp_svg.read_to_string()?;
    with_state(|state| {
        state.temp_svg = Some(temp_svg);
        state.saved_contents = Some(contents);
    });
    Ok(())
}

/// Clone the saved SVG contents recorded by the save step.
///
/// # Errors
///
/// Returns `Err` if the save step has not yet recorded the contents.
fn saved_contents() -> Result<String, TestSupportError> {
    with_state(|state| state.saved_contents.clone())
        .ok_or_else(|| TestSupportError::missing("saved SVG contents", "set by the save step"))
}

#[then("the saved SVG contains the known Gauss identifier")]
fn saved_svg_contains_id() -> Result<(), TestSupportError> {
    let expected_id = with_state(|state| state.expected_id).ok_or_else(|| {
        TestSupportError::missing("expected Gauss identifier", "set by the Given step")
    })?;
    let expected = format!(
        "gauss:id=\"{}\"",
        gauss::svg::metadata::shape_id_to_hex(expected_id)
    );
    let contents = saved_contents()?;
    if !contents.contains(&expected) {
        return Err(TestSupportError::expectation(format!(
            "saved SVG did not contain {expected}: {contents}"
        )));
    }
    Ok(())
}

#[then("the saved SVG contains a metadata element")]
fn saved_svg_contains_metadata_element() -> Result<(), TestSupportError> {
    if !saved_contents()?.contains("<metadata>") {
        return Err(TestSupportError::expectation(
            "saved SVG did not contain a <metadata> element",
        ));
    }
    Ok(())
}

#[then("the saved SVG contains the Gauss project metadata")]
fn saved_svg_contains_project_metadata() -> Result<(), TestSupportError> {
    if !saved_contents()?.contains("<gauss:project>test-project</gauss:project>") {
        return Err(TestSupportError::expectation(
            "saved SVG did not preserve the Gauss project metadata",
        ));
    }
    Ok(())
}

#[given("a temporary SVG with a known Gauss shape identifier")]
fn svg_with_known_id(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected_id = shape_id_from_seed(99_u128);
    let hex = gauss::svg::metadata::shape_id_to_hex(expected_id);
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="{GAUSS_METADATA_NAMESPACE}">
<path d="M 0 0 L 10 10" stroke="#000000" fill="none" gauss:id="{hex}" />
</svg>"##
    );
    prepare_open_svg(cx, "gauss-test-open-metadata-id", &svg)?;
    with_state(|state| state.expected_id = Some(expected_id));
    Ok(())
}

#[given("a temporary SVG with a Gauss project metadata block")]
fn svg_with_metadata(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="{GAUSS_METADATA_NAMESPACE}">
<metadata><gauss:project>test-project</gauss:project></metadata>
<path d="M 0 0 L 10 10" stroke="#000000" fill="none" gauss:id="ffffffff00000001" />
</svg>"##
    );
    prepare_open_svg(cx, "gauss-test-open-metadata-block", &svg)
}

#[when("the SVG is opened through the file dialog")]
fn open_through_dialog(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    shell()?.with_visual_cx(cx, |visual_cx, _entity| {
        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();
        Ok(())
    })?;
    cx.run_until_parked();
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

#[then("the document shape has the known Gauss identifier")]
fn document_shape_has_known_id(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = with_state(|state| state.expected_id).ok_or_else(|| {
        TestSupportError::missing("expected Gauss identifier", "set by the Given step")
    })?;
    let handles = shell()?;
    let actual = cx.read(|app| {
        handles
            .entity
            .read(app)
            .document()
            .shape_at(0)
            .map(|shape| shape.id)
    });
    if actual != Some(expected) {
        return Err(TestSupportError::expectation(format!(
            "expected opened shape id {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

#[then("the shell metadata contains the Gauss project element")]
fn shell_metadata_contains_project(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let handles = shell()?;
    let metadata = cx.read(|app| {
        handles
            .entity
            .read(app)
            .gauss_metadata_block()
            .map(str::to_owned)
    });
    if !metadata
        .as_deref()
        .is_some_and(|block| block.contains("<gauss:project>test-project</gauss:project>"))
    {
        return Err(TestSupportError::expectation(format!(
            "expected Gauss project metadata, found {metadata:?}"
        )));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/file_io_metadata_round_trip.feature",
    name = "Save preserves the Gauss shape identifier",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn save_preserves_shape_id(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_metadata_round_trip.feature",
    name = "Save preserves the Gauss metadata block",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn save_preserves_metadata(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_metadata_round_trip.feature",
    name = "Open restores the Gauss shape identifier",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_restores_shape_id(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/file_io_metadata_round_trip.feature",
    name = "Open restores the Gauss metadata block",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_restores_metadata(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
