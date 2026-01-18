//! GPUI headless integration tests for Gauss “Open…” wiring.
//!
//! Note: GPUI 0.2.2's test platform does not implement `prompt_for_paths`.
//! Phase 0 therefore routes “Open…” through `prompt_for_new_path` when the
//! `Phase0Shell` is constructed via `Phase0Shell::new_for_tests`.

use std::path::Path;

mod common;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use common::{TempFileGuard, ensure_initial_draw, init_test_app};
use gauss::ui::{OpenSvg, Phase0Shell};
use gpui::TestAppContext;
use uuid::Uuid;

#[gpui::test]
fn open_action_loads_selected_svg(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!("gauss-test-open-{}.svg", Uuid::new_v4()));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;
    dir.write(file_name.as_path(), svg.as_bytes())
        .expect("test SVG file should be writable");
    let cleanup = TempFileGuard::new_with_path(dir, file_name, svg_path);
    let Some(svg_path_ref) = cleanup.path() else {
        panic!("temp file path should be set");
    };

    assert!(
        !cx.did_prompt_for_new_path(),
        "No open prompt should be visible before triggering Open"
    );

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));

        ensure_initial_draw(visual_cx);
        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();

        view
    };
    cx.run_until_parked();

    assert!(
        cx.did_prompt_for_new_path(),
        "Open action should prompt for a path (test backend uses new-path prompt)"
    );

    cx.simulate_new_path_selection(|_directory: &Path| {
        Some(svg_path_ref.as_std_path().to_path_buf())
    });
    cx.run_until_parked();

    let opened = cx.read(|app| view.read(app).last_opened_path().map(Path::to_path_buf));
    assert_eq!(opened.as_deref(), Some(svg_path_ref.as_std_path()));

    let shape_count = cx.read(|app| view.read(app).document().shapes.len());
    assert_eq!(shape_count, 1);
}
