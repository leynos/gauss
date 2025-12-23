//! Gauss GPUI application entrypoint.
//!
//! Phase 0 goal: open a window with a minimal root view and prove GPUI +
//! `gpui-component` integration builds on the pinned toolchain.

use gauss::ui::{GaussRoot, Phase0Shell};
use gpui::Point;
use gpui::{
    App, AppContext as _, Application, TitlebarOptions, WindowDecorations, WindowOptions, px,
};

fn main() {
    Application::new().run(|app: &mut App| {
        gauss::ui::init(app);

        let window_options = WindowOptions {
            is_resizable: true,
            window_decorations: Some(WindowDecorations::Client),
            titlebar: Some(TitlebarOptions {
                title: Some("Gauss".into()),
                appears_transparent: true,
                traffic_light_position: Some(Point::new(px(10.0), px(10.0))),
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
