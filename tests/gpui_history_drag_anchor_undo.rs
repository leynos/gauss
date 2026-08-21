//! GPUI headless integration tests for Phase 0 anchor dragging.

#[path = "common/gpui_history_drag_anchor_undo.rs"]
mod common;
#[path = "gpui_history_bdd/drag_anchor.rs"]
mod drag_anchor;
#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open.rs"]
mod history_bdd_support_open;

use common::{
    CanvasDragScenario, assert_vec2_close, canvas_drag_scenario, draw_point, read_document,
    read_history_len, require_draw_shape, simulate_document_undo, simulate_escape,
};
use gauss::model::{Anchor, SelItem, Shape, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

fn draw_two_point_line_path(visual_cx: &mut VisualTestContext, scenario: CanvasDragScenario) {
    draw_point(visual_cx, scenario.first);
    draw_point(visual_cx, scenario.second);
}

fn first_two_anchors(shape: &Shape) -> TestSupportResult<(Anchor, Anchor)> {
    let first = shape
        .path
        .anchors
        .first()
        .cloned()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "first two anchors"))?;
    let second = shape
        .path
        .anchors
        .get(1)
        .cloned()
        .ok_or_else(|| TestSupportError::missing("anchor 1", "first two anchors"))?;
    Ok((first, second))
}

fn drag_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
    scenario: CanvasDragScenario,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(scenario.first, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selected_anchor = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.contains(&SelItem::Shape(shape_id)) {
        return Err(TestSupportError::expectation(format!(
            "expected anchor interaction to keep the shape selected; selection={selection:?}"
        )));
    }
    if !selection.contains(&selected_anchor) {
        return Err(TestSupportError::expectation(format!(
            "expected mouse down to select the first anchor; selection={selection:?}"
        )));
    }

    visual_cx.simulate_mouse_move(scenario.drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(scenario.drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

fn verify_anchor_moved(
    shape: &Shape,
    originals: &(Anchor, Anchor),
    delta: gauss::model::Vec2,
) -> TestSupportResult<()> {
    let moved = first_two_anchors(shape)?;
    assert_vec2_close(
        moved.0.pos,
        originals.0.pos.add(delta),
        "first anchor moved",
    )?;
    assert_vec2_close(moved.1.pos, originals.1.pos, "second anchor stable")?;
    if let Some(handle_out) = originals.0.handle_out {
        let moved_handle_out = moved
            .0
            .handle_out
            .ok_or_else(|| TestSupportError::missing("handle_out", "after drag"))?;
        assert_vec2_close(
            moved_handle_out,
            handle_out.add(delta),
            "first anchor handle_out moved",
        )?;
    }
    Ok(())
}

fn verify_anchor_restored(shape: &Shape, originals: &(Anchor, Anchor)) -> TestSupportResult<()> {
    let restored = first_two_anchors(shape)?;
    assert_vec2_close(restored.0.pos, originals.0.pos, "first anchor restored")?;
    assert_vec2_close(
        restored.1.pos,
        originals.1.pos,
        "second anchor still stable after undo",
    )
}
