//! GPUI headless integration test for toggling back to draw mode.
//!
//! Phase 0 uses `Escape` as the mode toggle:
//! - Draw → Manipulate: commit the current open path and enter manipulate mode.
//! - Manipulate → Draw: return to draw mode so clicks place points again.

use gauss::ui::Phase0Shell;
use gpui::{KeyDownEvent, Keystroke, Modifiers, TestAppContext, VisualTestContext, point, px};

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn simulate_key(visual_cx: &mut VisualTestContext, key: &str, modifiers: Modifiers) {
    visual_cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers,
            key: key.to_owned(),
            key_char: None,
        },
        is_held: false,
    });
    visual_cx.run_until_parked();
}

fn read_shape_count(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> usize {
    visual_cx.read(|app| view.read(app).document().shapes.len())
}

#[gpui::test]
fn escape_in_manipulate_returns_to_draw(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

    let click_point = point(bounds.origin.x + px(10.0), bounds.origin.y + px(10.0));

    // Arrange: force manipulate mode so a click does not place points.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let shapes_before = read_shape_count(visual_cx, &view);
    visual_cx.simulate_mouse_move(click_point, None, Modifiers::none());
    visual_cx.simulate_click(click_point, Modifiers::none());
    visual_cx.run_until_parked();

    let shapes_after_click_in_manipulate = read_shape_count(visual_cx, &view);
    assert_eq!(
        shapes_after_click_in_manipulate, shapes_before,
        "expected manipulate mode click to not create new shapes"
    );

    // Act: press escape to return to draw mode, and click again.
    simulate_key(visual_cx, "escape", Modifiers::none());
    visual_cx.simulate_mouse_move(click_point, None, Modifiers::none());
    visual_cx.simulate_click(click_point, Modifiers::none());
    visual_cx.run_until_parked();

    // Assert: draw mode click placed the first point of a new open path.
    let shapes_after_click_in_draw = read_shape_count(visual_cx, &view);
    assert_eq!(
        shapes_after_click_in_draw,
        shapes_before.saturating_add(1),
        "expected draw mode click to create a new shape"
    );
}
