//! Validate that the Phase 0 shell can request application quit.
//!
//! The GPUI test platform does not necessarily exit when `App::quit()` is
//! invoked, so the shell exposes a `did_request_quit` flag for stable headless
//! assertions.

use gauss::ui::Phase0Shell;
use gpui::{Modifiers, TestAppContext, point, px};

#[gpui::test]
fn clicking_quit_button_requests_quit(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();

    let Some(bounds) = visual_cx.debug_bounds("#quit-button") else {
        panic!("quit button should have debug bounds after drawing");
    };

    let position = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
    visual_cx.run_until_parked();

    let did_request_quit = visual_cx.read(|app| view.read(app).did_request_quit());
    assert!(
        did_request_quit,
        "clicking Quit should record that a quit request occurred"
    );
}
