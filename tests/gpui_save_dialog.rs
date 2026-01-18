//! GPUI headless integration tests for the Save action.
//!
//! These tests validate Save prompt wiring and SVG file output without needing
//! a real window manager.

use std::path::Path;

mod common;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use common::{TempFileGuard, ensure_initial_draw, init_test_app};
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
    let cleanup_dir = temp_dir_handle
        .try_clone()
        .expect("temp dir handle should be clonable");
    let _cleanup = TempFileGuard::new(cleanup_dir, file_name.clone());
    cx.simulate_new_path_selection(|_directory: &Path| Some(expected.as_std_path().to_path_buf()));
    cx.run_until_parked();

    let saved = cx.read(|app| view.read(app).last_saved_path().map(Path::to_path_buf));
    assert_eq!(saved.as_deref(), Some(expected.as_std_path()));

    let contents = temp_dir_handle
        .read_to_string(file_name.as_path())
        .expect("Saved SVG file should be readable");
    assert!(
        contents.contains(r#"<path d="M 10 10 L 90 10 L 90 90 L 10 90 Z""#),
        "Saved SVG should include the demo shape path"
    );
}
