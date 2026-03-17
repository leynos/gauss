//! GPUI headless integration tests for the Phase 0 mode indicator.
//!
//! The Phase 0 UI shows a mode indicator line (for example, `Mode: Draw (Line)`).
//! This test asserts the indicator reflects:
//! - initial draw mode state,
//! - `Tab` toggling the draw edge mode label, and
//! - manipulate mode not displaying an edge mode suffix.



use crate::common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;

#[gpui::test]
fn mode_indicator_reflects_tool_and_edge_mode(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let initial = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
    assert_eq!(initial, "Mode: Draw (Line)");

    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();

    let after_tab = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
    assert_eq!(after_tab, "Mode: Draw (Bezier (auto))");

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let after_switch = visual_cx.read(|app| view.read(app).mode_status_line_for_tests());
    assert_eq!(after_switch, "Mode: Manipulate");
}
