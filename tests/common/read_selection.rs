//! Selection snapshots for GPUI integration tests.

use gauss::{model::Selection, ui::Phase0Shell};
use gpui::{Entity, VisualTestContext};

pub fn read_selection(visual_cx: &VisualTestContext, view: &Entity<Phase0Shell>) -> Selection {
    visual_cx.read(|app| view.read(app).selection().clone())
}
