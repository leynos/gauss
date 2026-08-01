//! Manipulate-mode transition verification for GPUI integration tests.

use gauss::ui::Phase0Shell;
use gpui::{Entity, KeyDownEvent, Keystroke, Modifiers, Pixels, Point, VisualTestContext};

pub fn switch_to_manipulate_mode_and_verify(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
    click_point: Point<Pixels>,
) {
    visual_cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers::none(),
            key: "escape".to_owned(),
            key_char: None,
        },
        is_held: false,
    });
    visual_cx.run_until_parked();

    let shapes_after_escape = visual_cx.read(|app| view.read(app).document().len());
    visual_cx.simulate_mouse_move(click_point, None, Modifiers::none());
    visual_cx.simulate_click(click_point, Modifiers::none());
    visual_cx.run_until_parked();
    let shapes_after_click = visual_cx.read(|app| view.read(app).document().len());
    assert_eq!(
        shapes_after_click, shapes_after_escape,
        "escape should switch to manipulate mode, where clicks do not add points"
    );
}
