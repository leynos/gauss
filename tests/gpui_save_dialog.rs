//! GPUI headless integration tests for the Save action.
//!
//! These tests validate Save prompt wiring and SVG file output without needing
//! a real window manager.

use std::path::Path;

mod common;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use common::{TempFileGuard, ensure_initial_draw, init_test_app};
use gauss::model::{Gradient, GradientKind, GradientStop, LinearGradient, Paint, Vec2};
use gauss::ui::{Phase0Shell, SaveSvg};
use gpui::TestAppContext;
use uuid::Uuid;

#[gpui::test]
fn save_action_prompts_for_path(cx: &mut TestAppContext) {
    init_test_app(cx);

    assert!(
        !cx.did_prompt_for_new_path(),
        "No save prompt should be visible before triggering Save"
    );

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);
        visual_cx.dispatch_action(SaveSvg);
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();

    assert!(
        cx.did_prompt_for_new_path(),
        "Save action should prompt for a new path"
    );

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!("gauss-test-save-{}.svg", Uuid::new_v4()));
    let expected = temp_dir.join(&file_name);
    let temp_dir_handle =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let cleanup = TempFileGuard::new(temp_dir_handle, file_name.clone());
    cx.simulate_new_path_selection(|_directory: &Path| Some(expected.as_std_path().to_path_buf()));
    cx.run_until_parked();

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
}

#[gpui::test]
fn save_action_reports_dangling_resource_references(cx: &mut TestAppContext) {
    init_test_app(cx);

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);
        view.update(visual_cx, |shell, _cx| {
            let dangling_gradient = shell
                .resources_mut_for_tests()
                .insert_gradient(Gradient::new(
                    "dangling",
                    GradientKind::Linear(LinearGradient::new(
                        Vec2::new(0.0, 0.0),
                        Vec2::new(1.0, 0.0),
                        vec![
                            GradientStop::new(0.0, gauss::model::Rgba::new(255, 0, 0, 255)),
                            GradientStop::new(1.0, gauss::model::Rgba::new(255, 255, 0, 255)),
                        ],
                    )),
                ));
            let _removed = shell
                .resources_mut_for_tests()
                .remove_gradient(dangling_gradient);
            if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
                shape.style.fill = Paint::gradient(dangling_gradient);
            }
        });
        visual_cx.dispatch_action(SaveSvg);
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-save-dangling-resource-{}.svg",
        Uuid::new_v4()
    ));
    let expected = temp_dir.join(&file_name);
    let temp_dir_handle =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let cleanup = TempFileGuard::new(temp_dir_handle, file_name.clone());

    cx.simulate_new_path_selection(|_directory: &Path| Some(expected.as_std_path().to_path_buf()));
    cx.run_until_parked();

    let (saved, save_error) = cx.read(|app| {
        let shell = view.read(app);
        (
            shell.last_saved_path().map(Path::to_path_buf),
            shell.last_save_error().map(str::to_owned),
        )
    });

    assert!(
        saved.is_none(),
        "save path should not be recorded when export validation fails"
    );
    let error = save_error.expect("save error should be populated");
    assert!(
        error.contains("missing gradient resource"),
        "save error should report missing gradient references, got: {error}"
    );
    let read_result = cleanup.dir().read_to_string(file_name.as_path());
    assert!(
        read_result.is_err(),
        "save should not create file contents when validation fails"
    );
}
