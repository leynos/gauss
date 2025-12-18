//! Gauss GPUI application entrypoint.
//!
//! Phase 0 goal: open a window with a minimal root view and prove GPUI +
//! `gpui-component` integration builds on the pinned toolchain.

use gauss::ui::Phase0Shell;
use gpui::{App, AppContext as _, Application, TitlebarOptions, WindowDecorations, WindowOptions};
use gpui_component::Root;

fn main() {
    Application::new().run(|app: &mut App| {
        gpui_component::init(app);

        let window_options = WindowOptions {
            is_resizable: false,
            window_decorations: Some(WindowDecorations::Server),
            titlebar: Some(TitlebarOptions {
                title: Some("Gauss".into()),
                ..TitlebarOptions::default()
            }),
            ..WindowOptions::default()
        };

        if app
            .open_window(window_options, |window, cx| {
                let shell = cx.new(Phase0Shell::new);
                cx.new(|root_cx| Root::new(shell, window, root_cx))
            })
            .is_err()
        {
            app.quit();
        }
    });
}
