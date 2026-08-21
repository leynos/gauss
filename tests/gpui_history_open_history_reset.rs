//! BDD coverage for history and selection reset after opening a document.

#[path = "common/gpui_history_open_history_reset.rs"]
mod common;

#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_entity.rs"]
mod history_bdd_support_entity;
#[path = "gpui_history_bdd/support_open_for_tests.rs"]
mod history_bdd_support_open_for_tests;

use std::{cell::RefCell, path::Path};

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use common::{TempFileGuard, read_history_len, read_selection};
use gauss::model::{Command, SelItem, Selection, ShapeMovement, Vec2};
use gauss::ui::OpenSvg;
use gpui::TestAppContext;
use history_bdd_support::{DurableShell, missing};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use test_support::{TestSupportError, TestSupportResult};
use uuid::Uuid;

#[derive(Default)]
struct OpenState {
    shell: Option<DurableShell>,
    file: Option<TempFileGuard>,
}

thread_local! {
    static STATE: RefCell<OpenState> = RefCell::new(OpenState::default());
}

fn with_state<R>(f: impl FnOnce(&mut OpenState) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn reset_state() {
    with_state(|state| *state = OpenState::default());
}
struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        reset_state();
    }
}

#[fixture]
fn cleanup() -> Cleanup {
    reset_state();
    Cleanup
}
fn create_open_fixture() -> TestSupportResult<TempFileGuard> {
    let temp_dir = Utf8PathBuf::from_path_buf(std::env::temp_dir())
        .map_err(|_| TestSupportError::expectation("temp dir should be valid UTF-8"))?;
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-open-history-clear-{}.svg",
        Uuid::new_v4()
    ));
    let svg_path = temp_dir.join(&file_name);
    let dir = Dir::open_ambient_dir(&temp_dir, ambient_authority()).map_err(|error| {
        TestSupportError::expectation(format!("temp dir should be readable: {error}"))
    })?;
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg">
      <path d="M 8 8 L 24 24" stroke="#000000" stroke-width="1" fill="none" />
    </svg>"##;
    dir.write(file_name.as_path(), svg.as_bytes())
        .map_err(|error| {
            TestSupportError::expectation(format!("test SVG file should be writable: {error}"))
        })?;
    Ok(TempFileGuard::new(dir, file_name, svg_path))
}

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.shell.clone()).ok_or_else(|| missing("Phase 0 shell"))
}

#[given("a fresh Phase 0 shell test window with document history and selection")]
fn seeded_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state();
    let file = create_open_fixture()?;
    let shell = DurableShell::open_for_tests(cx);
    shell.with_visual(cx, |visual_cx, view| {
        let seed_result = visual_cx.update(|_window, app| -> TestSupportResult<()> {
            view.update(app, |phase0, _view_cx| {
                let shape_id = phase0
                    .document()
                    .shape_at(0)
                    .ok_or_else(|| {
                        TestSupportError::expectation("demo document should contain one shape")
                    })?
                    .id;
                let mut selection = Selection::default();
                selection.toggle(SelItem::Shape(shape_id));
                phase0.replace_selection_for_tests(selection);
                phase0
                    .apply_command_for_tests(Command::MoveShapes {
                        movements: vec![ShapeMovement {
                            shape_id,
                            delta: Vec2::new(5.0, 0.0),
                        }],
                    })
                    .map_err(|error| {
                        TestSupportError::expectation(format!(
                            "seeding history command should apply: {error}"
                        ))
                    })
            })
        });
        seed_result?;
        visual_cx.run_until_parked();
        if read_history_len(visual_cx, view) != 1 || read_selection(visual_cx, view).is_empty() {
            return Err(TestSupportError::expectation(
                "expected one history entry and a non-empty selection before open",
            ));
        }
        Ok(())
    })?;
    with_state(|state| {
        state.shell = Some(shell);
        state.file = Some(file);
    });
    Ok(())
}

#[when("another document is opened")]
fn open_document(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shell = shell()?;
    let path = with_state(|state| {
        state
            .file
            .as_ref()
            .map(TempFileGuard::path)
            .map(Utf8PathBuf::from)
    })
    .ok_or_else(|| missing("open fixture path"))?;
    shell.with_visual(cx, |visual_cx, _view| {
        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();
        Ok(())
    })?;
    cx.run_until_parked();
    cx.simulate_new_path_selection(|_directory: &Path| Some(path.as_std_path().to_path_buf()));
    cx.run_until_parked();
    Ok(())
}

#[then("the document history is empty")]
fn history_empty(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shell = shell()?;
    let actual = cx.read(|app| shell.entity().read(app).document_history_len_for_tests());
    if actual != 0 {
        return Err(TestSupportError::expectation(format!(
            "expected empty document history after open, found {actual} entries"
        )));
    }
    Ok(())
}

#[then("the selection is empty")]
fn selection_empty(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let shell = shell()?;
    let is_empty = cx.read(|app| shell.entity().read(app).selection().is_empty());
    if !is_empty {
        return Err(TestSupportError::expectation(
            "expected selection to be empty after open",
        ));
    }
    Ok(())
}

#[scenario(
    path = "tests/features/history_open_history_reset.feature",
    name = "Opening a document clears history and selection",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn open_history_reset_scenario(#[from(cleanup)] _cleanup: Cleanup) {}
