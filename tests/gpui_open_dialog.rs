//! GPUI headless integration tests for Gauss “Open…” wiring.
//!
//! Note: GPUI 0.2.2's test platform does not implement `prompt_for_paths`.
//! Phase 0 therefore routes “Open…” through `prompt_for_new_path` when the
//! `Phase0Shell` is constructed via `Phase0Shell::new_for_tests`.

use std::path::{Path, PathBuf};

mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::ui::{OpenSvg, Phase0Shell};
use gpui::TestAppContext;
use uuid::Uuid;

struct TempFileGuard {
    path: PathBuf,
}

impl TempFileGuard {
    const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = std::fs::remove_file(&self.path);
    }
}

#[gpui::test]
fn open_action_loads_selected_svg(cx: &mut TestAppContext) {
    init_test_app(cx);

    let cleanup = TempFileGuard::new(
        std::env::temp_dir().join(format!("gauss-test-open-{}.svg", Uuid::new_v4())),
    );
    let svg_path = cleanup.path();
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;
    std::fs::write(svg_path, svg).expect("Test SVG file should be writable");

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

    cx.simulate_new_path_selection(|_directory: &Path| Some(svg_path.to_path_buf()));
    cx.run_until_parked();

    let opened = cx.read(|app| view.read(app).last_opened_path().map(Path::to_path_buf));
    assert_eq!(opened.as_deref(), Some(svg_path));

    let shape_count = cx.read(|app| view.read(app).document().shapes.len());
    assert_eq!(shape_count, 1);
}
