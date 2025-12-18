//! Validate that headless GPUI tests can drive `on_click` handlers.

use gauss::ui::Phase0Shell;
use gpui::{Modifiers, TestAppContext, point, px};

#[gpui::test]
fn clicking_save_button_opens_save_prompt(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    assert!(
        !cx.did_prompt_for_new_path(),
        "save prompt should not be open before clicking Save…"
    );

    {
        let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        visual_cx.update(|window, app| drop(window.draw(app)));
        visual_cx.run_until_parked();

        let Some(bounds) = visual_cx.debug_bounds("#save-button") else {
            panic!("save button should have debug bounds after drawing");
        };
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
