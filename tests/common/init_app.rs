//! Application initialization for GPUI integration tests.

use gpui::TestAppContext;

pub fn init_test_app(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);
}
