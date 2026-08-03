//! Application initialization for GPUI integration tests.

use gpui::TestAppContext;

/// Initializes the Gauss application inside a GPUI test context.
pub fn init_test_app(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);
}
