//! Gauss GPUI application entrypoint.
//!
//! Phase 0 goal: open a window with a minimal root view and prove GPUI +
//! `gpui-component` integration builds on the pinned toolchain.

use gauss::ui::Phase0Shell;
use gpui::{App, AppContext as _, Application, WindowOptions};
use gpui_component::Root;

fn main() {
    Application::new().run(|app: &mut App| {
        gpui_component::init(app);

        if app
            .open_window(WindowOptions::default(), |window, cx| {
                let shell = cx.new(Phase0Shell::new);
                cx.new(|root_cx| Root::new(shell, window, root_cx))
            })
            .is_err()
        {
            app.quit();
        }
    });
}
