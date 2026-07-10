//! Structural GPUI coverage for the Phase 1 shell chrome layout.
//!
//! This raw test intentionally checks only that chrome elements participate in
//! layout. User interactions with those elements live in the BDD companion.

#[path = "common/gpui_shell_chrome_layout.rs"]
mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;

const CHROME_SELECTORS: [&str; 16] = [
    "#open-button",
    "#save-button",
    "#undo-button",
    "#redo-button",
    "#settings-button",
    "#window-minimize",
    "#window-maximize",
    "#quit-button",
    "#status-bar",
    "#align-left",
    "#align-center",
    "#align-right",
    "#zoom-out",
    "#zoom-in",
    "#zoom-area",
    "#snap-to-grid",
];

#[gpui::test]
fn chrome_layout_exposes_core_controls(cx: &mut TestAppContext) {
    init_test_app(cx);
    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    for selector in CHROME_SELECTORS {
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "expected chrome element bounds for {selector}"
        );
    }
}
