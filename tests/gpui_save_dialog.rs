//! GPUI headless integration tests for the Save action.
//!
//! These tests validate Save prompt wiring and SVG file output without needing
//! a real window manager.

use std::path::Path;

mod common;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use common::{TempFileGuard, ensure_initial_draw, init_test_app};
use gauss::model::{
    Gradient, GradientKind, GradientStop, LinearGradient, Paint, PatternResource, Rgba, Vec2,
};
use gauss::svg::metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX};
use gauss::ui::{ExportSvgWebReady, Phase0Shell, SaveSvg};
use gpui::TestAppContext;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExportAction {
    Save,
    WebReady,
}

fn setup_export_test_view(
    cx: &mut TestAppContext,
    mut configure_shell: impl FnMut(&mut Phase0Shell),
    action: ExportAction,
) -> gpui::Entity<Phase0Shell> {
    let view = {
        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);
        view.update(visual_cx, |shell, _cx| {
            configure_shell(shell);
        });
        match action {
            ExportAction::Save => visual_cx.dispatch_action(SaveSvg),
            ExportAction::WebReady => visual_cx.dispatch_action(ExportSvgWebReady),
        }
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();
    view
}

fn create_temp_save_target(
    prefix: &str,
) -> Result<(Utf8PathBuf, Utf8PathBuf, TempFileGuard), String> {
    let temp_dir = Utf8PathBuf::from_path_buf(std::env::temp_dir())
        .map_err(|_| "temp dir should be valid UTF-8".to_owned())?;
    let file_name = Utf8PathBuf::from(format!("{prefix}-{}.svg", Uuid::new_v4()));
    let expected = temp_dir.join(&file_name);
    let temp_dir_handle = Dir::open_ambient_dir(&temp_dir, ambient_authority())
        .map_err(|err| format!("temp dir should be readable: {err}"))?;
    let cleanup = TempFileGuard::new(temp_dir_handle, file_name.clone());
    Ok((expected, file_name, cleanup))
}

fn create_temp_save_target_or_panic(prefix: &str) -> (Utf8PathBuf, Utf8PathBuf, TempFileGuard) {
    match create_temp_save_target(prefix) {
        Ok(target) => target,
        Err(err) => panic!("{err}"),
    }
}

fn choose_save_path(cx: &mut TestAppContext, expected: &Utf8PathBuf) {
    cx.simulate_new_path_selection(|_directory: &Path| Some(expected.as_std_path().to_path_buf()));
    cx.run_until_parked();
}

fn read_save_outcome(
    cx: &TestAppContext,
    view: &gpui::Entity<Phase0Shell>,
) -> (Option<std::path::PathBuf>, Option<String>) {
    cx.read(|app| {
        let shell = view.read(app);
        (
            shell.last_saved_path().map(Path::to_path_buf),
            shell.last_save_error().map(str::to_owned),
        )
    })
}

fn assert_web_ready_contents(contents: &str) {
    assert!(
        contents.contains(r#"<path d="M 10 10 L 90 10 L 90 90 L 10 90 Z""#),
        "Web-ready SVG should include the demo shape path"
    );
    assert!(
        !contents.contains(GAUSS_METADATA_NAMESPACE),
        "Web-ready SVG must not include the Gauss namespace declaration"
    );
    assert!(
        !contents.contains(&format!("{GAUSS_METADATA_PREFIX}:")),
        "Web-ready SVG must not include namespaced Gauss metadata attributes"
    );
    assert!(
        !contents.contains("<metadata>"),
        "Web-ready SVG must not include metadata blocks"
    );
}

fn setup_dangling_gradient(shell: &mut Phase0Shell) {
    let dangling_gradient = shell
        .resources_mut_for_tests()
        .insert_gradient(Gradient::new(
            "dangling",
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                vec![
                    GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                    GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
                ],
            )),
        ));
    let _removed = shell
        .resources_mut_for_tests()
        .remove_gradient(dangling_gradient);
    if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
        shape.style.fill = Paint::gradient(dangling_gradient);
    }
}

fn assert_dangling_resource_validation(
    cx: &mut TestAppContext,
    action: ExportAction,
    test_prefix: &str,
) {
    let (saved_path_message, gradient_error_message, read_error_message) = match action {
        ExportAction::Save => (
            "save path should not be recorded when export validation fails",
            "save error should report missing gradient references, got: {error}",
            "save should not create file contents when validation fails",
        ),
        ExportAction::WebReady => (
            "export path should not be recorded when export validation fails",
            "web-ready export should report missing gradient references, got: {error}",
            "web-ready export should not create file contents when validation fails",
        ),
    };

    let view = setup_export_test_view(cx, setup_dangling_gradient, action);
    let (expected, file_name, cleanup) = create_temp_save_target_or_panic(test_prefix);
    choose_save_path(cx, &expected);

    let (saved, save_error) = read_save_outcome(cx, &view);
    assert!(saved.is_none(), "{saved_path_message}");
    let Some(error) = save_error else {
        panic!("save error should be populated");
    };
    assert!(
        error.contains("missing gradient resource"),
        "{gradient_error_message}"
    );
    let read_result = cleanup.dir().read_to_string(file_name.as_path());
    assert!(read_result.is_err(), "{read_error_message}");
}

#[gpui::test]
fn save_action_prompts_for_path(cx: &mut TestAppContext) {
    init_test_app(cx);

    assert!(
        !cx.did_prompt_for_new_path(),
        "No save prompt should be visible before triggering Save"
    );

    let view = setup_export_test_view(cx, |_shell| {}, ExportAction::Save);

    assert!(
        cx.did_prompt_for_new_path(),
        "Save action should prompt for a new path"
    );

    let (expected, file_name, cleanup) = create_temp_save_target_or_panic("gauss-test-save");
    choose_save_path(cx, &expected);

    let saved = cx.read(|app| view.read(app).last_saved_path().map(Path::to_path_buf));
    assert_eq!(saved.as_deref(), Some(expected.as_std_path()));

    let contents = cleanup
        .dir()
        .read_to_string(file_name.as_path())
        .expect("Saved SVG file should be readable");
    assert!(
        contents.contains(r#"<path d="M 10 10 L 90 10 L 90 90 L 10 90 Z""#),
        "Saved SVG should include the demo shape path"
    );
    let expected_namespace =
        format!(r#"xmlns:{GAUSS_METADATA_PREFIX}="{GAUSS_METADATA_NAMESPACE}""#);
    assert!(
        contents.contains(expected_namespace.as_str()),
        "Saved SVG should include canonical Gauss metadata namespace declaration"
    );
}

#[gpui::test]
fn save_action_reports_dangling_resource_references(cx: &mut TestAppContext) {
    init_test_app(cx);
    assert_dangling_resource_validation(
        cx,
        ExportAction::Save,
        "gauss-test-save-dangling-resource",
    );
}

#[gpui::test]
fn save_action_reports_dangling_pattern_references(cx: &mut TestAppContext) {
    init_test_app(cx);

    let view = setup_export_test_view(
        cx,
        |shell| {
            let dangling_pattern = shell
                .resources_mut_for_tests()
                .insert_pattern(PatternResource::new("dangling-pattern", "<circle />"));
            let _removed = shell
                .resources_mut_for_tests()
                .remove_pattern(dangling_pattern);
            if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
                shape.style.fill = Paint::pattern(dangling_pattern);
            }
        },
        ExportAction::Save,
    );
    let (expected, file_name, cleanup) =
        create_temp_save_target_or_panic("gauss-test-save-dangling-pattern");
    choose_save_path(cx, &expected);

    let (saved, save_error) = read_save_outcome(cx, &view);
    assert!(
        saved.is_none(),
        "save path should not be recorded when export validation fails"
    );
    let Some(error) = save_error else {
        panic!("save error should be populated");
    };
    assert!(
        error.contains("missing pattern resource"),
        "save error should report missing pattern references, got: {error}"
    );
    let read_result = cleanup.dir().read_to_string(file_name.as_path());
    assert!(
        read_result.is_err(),
        "save should not create file contents when validation fails"
    );
}

#[gpui::test]
fn web_ready_export_action_strips_gauss_metadata(cx: &mut TestAppContext) {
    init_test_app(cx);

    assert!(
        !cx.did_prompt_for_new_path(),
        "No export prompt should be visible before triggering web-ready export"
    );

    let view = setup_export_test_view(
        cx,
        |shell| {
            shell.set_gauss_metadata_block_for_tests(Some(
                "<gauss:test gauss:purpose=\"round-trip\" />".to_owned(),
            ));
        },
        ExportAction::WebReady,
    );

    assert!(
        cx.did_prompt_for_new_path(),
        "Web-ready export action should prompt for a new path"
    );

    let (expected, file_name, cleanup) =
        create_temp_save_target_or_panic("gauss-test-web-ready-export");
    choose_save_path(cx, &expected);

    let (saved, save_error) = read_save_outcome(cx, &view);
    assert_eq!(saved.as_deref(), Some(expected.as_std_path()));
    assert!(
        save_error.is_none(),
        "web-ready export should not set save errors on success"
    );

    let contents = cleanup
        .dir()
        .read_to_string(file_name.as_path())
        .expect("Web-ready SVG file should be readable");
    assert_web_ready_contents(&contents);
}

#[gpui::test]
fn web_ready_export_reports_dangling_resource_references(cx: &mut TestAppContext) {
    init_test_app(cx);
    assert_dangling_resource_validation(
        cx,
        ExportAction::WebReady,
        "gauss-test-web-ready-dangling-resource",
    );
}
