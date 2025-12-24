//! Gauss GPUI application entrypoint.
//!
//! Phase 0 goal: open a window with a minimal root view and prove GPUI +
//! `gpui-component` integration builds on the pinned toolchain.

use gauss::ui::{GaussRoot, Phase0Shell};
use gpui::Point;
use gpui::{
    App, AppContext as _, Application, Pixels, TitlebarOptions, WindowDecorations, WindowOptions,
    px,
};

/// Horizontal offset for macOS traffic light buttons from the left edge.
const TRAFFIC_LIGHT_X_OFFSET: Pixels = px(10.0);

/// Vertical offset for macOS traffic light buttons from the top edge.
const TRAFFIC_LIGHT_Y_OFFSET: Pixels = px(10.0);

fn main() {
    Application::new().run(|app: &mut App| {
        gauss::ui::init(app);

        let window_options = WindowOptions {
            is_resizable: true,
            window_decorations: Some(WindowDecorations::Client),
            titlebar: Some(TitlebarOptions {
                title: Some("Gauss".into()),
                appears_transparent: true,
                traffic_light_position: Some(Point::new(
                    TRAFFIC_LIGHT_X_OFFSET,
                    TRAFFIC_LIGHT_Y_OFFSET,
                )),
            }),
            ..WindowOptions::default()
        };

        // Use our custom GaussRoot that respects maximized state.
        // gpui-component's Root uses window_border() which doesn't check
        // is_maximized() before setting up resize zones, blocking clicks.
        if app
            .open_window(window_options, |_window, cx| {
                let shell = cx.new(Phase0Shell::new);
                cx.new(|ctx| GaussRoot::new(shell, ctx))
            })
            .is_err()
        {
            app.quit();
        }
    });
}
