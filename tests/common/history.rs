//! Document-history reads for GPUI integration tests.

use gauss::ui::Phase0Shell;
use gpui::{Entity, VisualTestContext};

pub fn read_history_len(visual_cx: &VisualTestContext, view: &Entity<Phase0Shell>) -> usize {
    visual_cx.read(|app| view.read(app).document_history_len_for_tests())
}
