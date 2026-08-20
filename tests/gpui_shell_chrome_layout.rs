//! Behavioural tests for the Phase 1 chrome layout.

#[path = "common/gpui_shell_chrome_layout.rs"]
mod common;

use common::{canvas_bounds, click_canvas_and_wait, ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, point, px};

const CHROME_SELECTORS: [&str; 16] = [
    "#open-button",
    "#save-button",
    "#undo-button",
    "#redo-button",
    "#settings-button",
    "#window-minimize",
    "#window-maximize",
    "#quit-button",
    "#status-bar",
    "#align-left",
    "#align-center",
    "#align-right",
    "#zoom-out",
    "#zoom-in",
    "#zoom-area",
    "#snap-to-grid",
];

#[gpui::test]
fn chrome_layout_exposes_core_controls(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    for selector in CHROME_SELECTORS {
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "expected chrome element bounds for {selector}"
        );
    }
}

#[gpui::test]
fn chrome_layout_keeps_canvas_clicks_working(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let position = point(bounds.origin.x + px(4.0), bounds.origin.y + px(4.0));
    click_canvas_and_wait(visual_cx, position);

    let click = visual_cx.read(|app| view.read(app).last_canvas_click_screen());
    assert!(
        click.is_some(),
        "expected the canvas to still record click locations after chrome layout"
    );
}

#[gpui::test]
fn chrome_open_button_requests_path(cx: &mut TestAppContext) {
    init_test_app(cx);

    assert!(
        !cx.did_prompt_for_new_path(),
        "no open prompt should be visible before triggering Open"
    );

    let (_view, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = visual_cx
        .debug_bounds("#open-button")
        .expect("expected open button bounds");
    let position = point(bounds.origin.x + px(4.0), bounds.origin.y + px(4.0));
    click_canvas_and_wait(visual_cx, position);
    cx.run_until_parked();

    assert!(
        cx.did_prompt_for_new_path(),
        "open button should request a file path"
    );
    cx.simulate_new_path_selection(|_directory| None);
    cx.run_until_parked();
}

#[gpui::test]
fn chrome_save_button_requests_path(cx: &mut TestAppContext) {
    init_test_app(cx);

    assert!(
        !cx.did_prompt_for_new_path(),
        "no save prompt should be visible before triggering Save"
    );

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = visual_cx
        .debug_bounds("#save-button")
        .expect("expected save button bounds");
    let position = point(bounds.origin.x + px(4.0), bounds.origin.y + px(4.0));
    click_canvas_and_wait(visual_cx, position);
    cx.run_until_parked();

    assert!(
        cx.did_prompt_for_new_path(),
        "save button should request a file path"
    );
    cx.simulate_new_path_selection(|_directory| None);
    cx.run_until_parked();
}

#[gpui::test]
fn chrome_undo_redo_buttons_update_document(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let p1 = point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0));
    let p2 = point(bounds.origin.x + px(80.0), bounds.origin.y + px(24.0));

    click_canvas_and_wait(visual_cx, p1);
    click_canvas_and_wait(visual_cx, p2);

    let before = visual_cx.read(|app| view.read(app).document().clone());
    let shape_before = common::require_draw_shape(&before, "after drawing")
        .expect("expected draw shape after drawing");
    let anchor_count = shape_before.path.anchors.len();
    assert!(
        anchor_count > 1,
        "expected at least two anchors before undo"
    );

    let undo_bounds = visual_cx
        .debug_bounds("#undo-button")
        .expect("expected undo button bounds");
    let undo_pos = point(
        undo_bounds.origin.x + px(4.0),
        undo_bounds.origin.y + px(4.0),
    );
    click_canvas_and_wait(visual_cx, undo_pos);

    let after_undo = visual_cx.read(|app| view.read(app).document().clone());
    let shape_after_undo = common::require_draw_shape(&after_undo, "after undo")
        .expect("expected draw shape after undo");
    assert_eq!(
        shape_after_undo.path.anchors.len(),
        anchor_count - 1,
        "expected undo to remove the last anchor"
    );

    let redo_bounds = visual_cx
        .debug_bounds("#redo-button")
        .expect("expected redo button bounds");
    let redo_pos = point(
        redo_bounds.origin.x + px(4.0),
        redo_bounds.origin.y + px(4.0),
    );
    click_canvas_and_wait(visual_cx, redo_pos);

    let after_redo = visual_cx.read(|app| view.read(app).document().clone());
    let shape_after_redo = common::require_draw_shape(&after_redo, "after redo")
        .expect("expected draw shape after redo");
    assert_eq!(
        shape_after_redo.path.anchors.len(),
        anchor_count,
        "expected redo to restore the anchor"
    );
}

#[gpui::test]
fn chrome_quit_button_sets_quit_flag(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = visual_cx
        .debug_bounds("#quit-button")
        .expect("expected quit button bounds");
    let position = point(bounds.origin.x + px(4.0), bounds.origin.y + px(4.0));
    click_canvas_and_wait(visual_cx, position);

    let did_quit = visual_cx.read(|app| view.read(app).did_request_quit());
    assert!(did_quit, "expected quit button to request quit");
}
