//! Behavioural tests for the Phase 1 tool rail.

#[path = "common/gpui_shell_tool_rail.rs"]
mod common;

use common::{click_left_and_wait, ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, point, px};

fn click_tool(visual_cx: &mut gpui::VisualTestContext, selector: &'static str) {
    let Some(bounds) = visual_cx.debug_bounds(selector) else {
        panic!("missing bounds for {selector}");
    };
    let position = point(bounds.origin.x + px(4.0), bounds.origin.y + px(4.0));
    click_left_and_wait(visual_cx, position);
}

#[gpui::test]
fn tool_rail_select_enters_manipulate_and_clears_active_shape(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            let demo_id = shell
                .document()
                .shape_id_at(0)
                .expect("demo document should include a shape");
            shell.set_draw_active_shape_for_tests(Some(demo_id));
        });
    });
    visual_cx.run_until_parked();

    click_tool(visual_cx, "#tool-select");

    let mode = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
    assert_eq!(mode, "Mode: Manipulate");

    let active_shape = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert!(
        active_shape.is_none(),
        "expected draw active shape to clear"
    );
}

#[gpui::test]
fn tool_rail_draw_buttons_switch_edge_modes(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    click_tool(visual_cx, "#tool-draw-curve");
    let mode = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
    assert_eq!(mode, "Mode: Draw (Bezier (auto))");

    click_tool(visual_cx, "#tool-draw-line");
    let mode_after = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
    assert_eq!(mode_after, "Mode: Draw (Line)");
}
