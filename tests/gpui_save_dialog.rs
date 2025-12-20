//! GPUI headless integration tests for the Save action.
//!
//! These tests validate Save prompt wiring and SVG file output without needing
//! a real window manager.

use std::path::Path;

mod common;

use common::{ensure_initial_draw, init_test_app};
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

    let expected = std::env::temp_dir().join(format!("gauss-test-save-{}.svg", Uuid::new_v4()));
    cx.simulate_new_path_selection(|_directory: &Path| Some(expected.clone()));
    cx.run_until_parked();

    let saved = cx.read(|app| view.read(app).last_saved_path().map(Path::to_path_buf));
    assert_eq!(saved, Some(expected.clone()));

    let contents = std::fs::read_to_string(&expected).expect("Saved SVG file should be readable");
    assert!(
        contents.contains(r#"<path d="M 10 10 L 90 10 L 90 90 L 10 90 Z""#),
        "Saved SVG should include the demo shape path"
    );
}
