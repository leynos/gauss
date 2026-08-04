//! Selection reads for GPUI integration tests.

use gauss::{model::SelItem, ui::Phase0Shell};
use gpui::{Entity, VisualTestContext};

/// Returns a copied list of the shell view's current
/// [`gauss::model::Selection::items`].
pub fn read_selection_items(
    visual_cx: &VisualTestContext,
    view: &Entity<Phase0Shell>,
) -> Vec<SelItem> {
    visual_cx.read(|app| view.read(app).selection().items.clone())
}
