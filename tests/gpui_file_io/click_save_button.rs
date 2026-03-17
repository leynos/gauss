//! Validate that headless GPUI tests can drive the Save button `on_click`
//! handler and observe the platform save prompt state.

use crate::crate::common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, TestAppContext, point, px};

#[gpui::test]
fn clicking_save_button_opens_save_prompt(cx: &mut TestAppContext) {
    init_test_app(cx);

    assert!(
        !cx.did_prompt_for_new_path(),
        "save prompt should not be open before clicking Save…"
    );

    {
        let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);

        let bounds = visual_cx
            .debug_bounds("#save-button")
            .expect("save button should have debug bounds after drawing");
        let position = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
        visual_cx.simulate_mouse_move(position, None, Modifiers::none());
        visual_cx.simulate_click(position, Modifiers::none());
        visual_cx.run_until_parked();
    }
    cx.run_until_parked();

    assert!(
        cx.did_prompt_for_new_path(),
        "clicking Save… should trigger the platform save prompt"
    );
}
