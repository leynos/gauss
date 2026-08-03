//! Selection snapshots for GPUI integration tests.

use gauss::{model::Selection, ui::Phase0Shell};
use gpui::{Entity, VisualTestContext};

/// Returns an owned snapshot of the shell view's current selection.
pub fn read_selection(visual_cx: &VisualTestContext, view: &Entity<Phase0Shell>) -> Selection {
    visual_cx.read(|app| view.read(app).selection().clone())
}
