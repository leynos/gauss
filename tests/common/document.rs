//! Document reads for GPUI integration tests.

use gauss::{model::Document, ui::Phase0Shell};
use gpui::{Entity, VisualTestContext};

pub fn read_document(visual_cx: &VisualTestContext, view: &Entity<Phase0Shell>) -> Document {
    visual_cx.read(|app| view.read(app).document().clone())
}
