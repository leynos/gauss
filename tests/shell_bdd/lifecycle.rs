//! Focused GPUI lifecycle helpers for shell BDD scenario binaries.
//!
//! Shell BDD binaries need application initialisation and one completed initial
//! draw, but do not need the broad legacy `common` helper surface.

use gpui::{TestAppContext, VisualTestContext};

/// Initialise the application before constructing a shell under test.
pub fn init_test_app(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);
}

/// Complete the initial draw so selector and event helpers see a settled view.
pub fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| {
        let _draw = window.draw(app);
    });
    visual_cx.run_until_parked();
}
