//! GPUI headless integration test verifying that closing a path produces
//! exactly one undo entry and that undo reopens it.

#[path = "gpui_history_bdd/close_path.rs"]
mod close_path;

#[path = "common/gpui_history_close_path_undo.rs"]
mod common;

#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open.rs"]
mod history_bdd_support_open;

use common::{
    anchor_to_canvas_point, canvas_bounds, draw_point, read_document, read_history_len,
    require_draw_shape, simulate_document_undo,
};
use gpui::{Bounds, Pixels, point, px};
fn triangle_points(
    bounds: &Bounds<Pixels>,
) -> (
    gpui::Point<Pixels>,
    gpui::Point<Pixels>,
    gpui::Point<Pixels>,
) {
    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + px(12.0),
    );
    let p3 = point(
        bounds.origin.x + bounds.size.width - px(12.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    (p1, p2, p3)
}
