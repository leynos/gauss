//! Selection reads for GPUI integration tests.

use gauss::{model::SelItem, ui::Phase0Shell};
use gpui::{Entity, VisualTestContext};

pub fn read_selection_items(
    visual_cx: &VisualTestContext,
    view: &Entity<Phase0Shell>,
) -> Vec<SelItem> {
    visual_cx.read(|app| view.read(app).selection().items.clone())
}
