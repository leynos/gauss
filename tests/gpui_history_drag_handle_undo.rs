//! GPUI headless integration tests for Phase 0 handle dragging.

#[path = "common/gpui_history_drag_handle_undo.rs"]
mod common;
#[path = "gpui_history_bdd/drag_handle.rs"]
mod drag_handle;
#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open.rs"]
mod history_bdd_support_open;

use common::{
    CanvasDragScenario, anchor_to_canvas_point, assert_vec2_close, canvas_drag_scenario,
    read_document, read_history_len, require_draw_shape, simulate_document_undo, simulate_escape,
};
use gauss::model::{SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

fn toggle_bezier_auto(visual_cx: &mut VisualTestContext) {
    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();
}

fn draw_two_point_bezier_path(visual_cx: &mut VisualTestContext, scenario: CanvasDragScenario) {
    toggle_bezier_auto(visual_cx);
    visual_cx.simulate_mouse_move(scenario.first, None, Modifiers::none());
    visual_cx.simulate_click(scenario.first, Modifiers::none());
    visual_cx.simulate_mouse_move(scenario.second, None, Modifiers::none());
    visual_cx.simulate_click(scenario.second, Modifiers::none());
    visual_cx.run_until_parked();
}

#[derive(Clone, Copy, Debug)]
struct HandleDragSetup {
    scenario: CanvasDragScenario,
    shape_id: ShapeId,
    first_anchor_pos: Vec2,
    original_handle_out: Vec2,
    handle_start: gpui::Point<gpui::Pixels>,
    handle_end: gpui::Point<gpui::Pixels>,
}

fn setup_handle_drag(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<HandleDragSetup> {
    let scenario = canvas_drag_scenario(visual_cx, 18.0, 10.0)?;
    draw_two_point_bezier_path(visual_cx, scenario);

    let doc_before = read_document(visual_cx, view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points")?;
    let first_anchor = original_shape
        .path
        .anchors
        .first()
        .cloned()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after drawing"))?;
    let original_handle_out = first_anchor.handle_out.ok_or_else(|| {
        TestSupportError::missing("handle_out", "after drawing bezier auto shape")
    })?;

    let handle_start =
        anchor_to_canvas_point(&scenario.bounds, original_handle_out, scenario.first);
    let handle_end = point(
        handle_start.x + px(scenario.delta.x),
        handle_start.y + px(scenario.delta.y),
    );

    simulate_escape(visual_cx);

    Ok(HandleDragSetup {
        scenario,
        shape_id: original_shape.id,
        first_anchor_pos: first_anchor.pos,
        original_handle_out,
        handle_start,
        handle_end,
    })
}

fn assert_handle_selection_includes_shape(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.contains(&SelItem::Shape(shape_id)) {
        return Err(TestSupportError::expectation(format!(
            "expected handle interaction to keep the shape selected; selection={selection:?}"
        )));
    }
    if !selection.contains(&SelItem::HandleOut {
        shape: shape_id,
        anchor: 0,
    }) {
        return Err(TestSupportError::expectation(format!(
            "expected mouse down to select the handle; selection={selection:?}"
        )));
    }
    Ok(())
}
