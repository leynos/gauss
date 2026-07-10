//! GPUI headless integration tests for Phase 0 z-order reordering.
//!
//! This test draws two identical (overlapping) shapes so that “raise” and
//! “lower” behaviour depends only on z-order, not on choosing distinct hit
//! targets.

#[path = "common/gpui_history_reorder_undo.rs"]
mod common;
#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/reorder.rs"]
mod reorder;

use common::{
    click_canvas_and_wait, demo_shape_id, read_document, read_selection, simulate_escape,
};
use gauss::model::{Document, SelItem, Selection, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Clone, Copy, Debug)]
struct LinePoints {
    start: gpui::Point<gpui::Pixels>,
    end: gpui::Point<gpui::Pixels>,
}

fn selected_shape_id(selection: &Selection) -> Option<ShapeId> {
    let item = selection.items.first()?;
    Some(match item {
        SelItem::Shape(shape)
        | SelItem::Anchor { shape, .. }
        | SelItem::HandleIn { shape, .. }
        | SelItem::HandleOut { shape, .. }
        | SelItem::Segment { shape, .. } => *shape,
    })
}

fn require_drawn_shape_ids(doc: &Document) -> TestSupportResult<(ShapeId, ShapeId)> {
    let demo_id = demo_shape_id(doc)
        .ok_or_else(|| TestSupportError::missing("demo shape", "after drawing"))?;
    let mut ids = doc
        .iter_ids_in_draw_order()
        .filter(|shape_id| *shape_id != demo_id);

    let first = ids
        .next()
        .ok_or_else(|| TestSupportError::missing("drawn shape 1", "after drawing"))?;
    let second = ids
        .next()
        .ok_or_else(|| TestSupportError::missing("drawn shape 2", "after drawing"))?;
    if ids.next().is_some() {
        return Err(TestSupportError::expectation(
            "expected exactly two drawn shapes",
        ));
    }
    Ok((first, second))
}

fn require_sorted_drawn_shape_ids(doc: &Document) -> TestSupportResult<Vec<ShapeId>> {
    let (first, second) = require_drawn_shape_ids(doc)?;
    let mut ids = vec![first, second];
    ids.sort_by_key(|id| id.to_accesskit_node_id());
    Ok(ids)
}

fn require_shape_index(
    doc: &Document,
    shape_id: ShapeId,
    context: &str,
) -> TestSupportResult<usize> {
    doc.find_index(shape_id)
        .ok_or_else(|| TestSupportError::missing("shape index", format!("{shape_id:?}: {context}")))
}

fn ordered_pair(
    doc: &Document,
    a: ShapeId,
    b: ShapeId,
    context: &str,
) -> TestSupportResult<(ShapeId, ShapeId)> {
    let a_index = require_shape_index(doc, a, context)?;
    let b_index = require_shape_index(doc, b, context)?;
    Ok(if a_index <= b_index { (a, b) } else { (b, a) })
}

fn assert_relative_order(
    doc: &Document,
    lower: ShapeId,
    higher: ShapeId,
    context: &str,
) -> TestSupportResult<()> {
    let lower_index = doc
        .find_index(lower)
        .ok_or_else(|| TestSupportError::missing("shape index", format!("{lower:?}: {context}")))?;
    let higher_index = doc.find_index(higher).ok_or_else(|| {
        TestSupportError::missing("shape index", format!("{higher:?}: {context}"))
    })?;
    if lower_index >= higher_index {
        return Err(TestSupportError::expectation(format!(
            "expected {lower:?} below {higher:?}: {context}"
        )));
    }
    Ok(())
}

fn line_points(bounds: &gpui::Bounds<gpui::Pixels>) -> LinePoints {
    let start = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let end = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + px(2.0),
    );
    LinePoints { start, end }
}

fn draw_overlapping_lines(visual_cx: &mut VisualTestContext, points: LinePoints) {
    click_canvas_and_wait(visual_cx, points.start);
    click_canvas_and_wait(visual_cx, points.end);
    simulate_escape(visual_cx);

    // First escape commits the open path so it becomes a selectable shape.
    simulate_escape(visual_cx);
    click_canvas_and_wait(visual_cx, points.start);
    click_canvas_and_wait(visual_cx, points.end);
    // Second escape clears selection state before we click to reorder.
    simulate_escape(visual_cx);
}

fn verify_initial_shapes_and_order(
    doc: &Document,
) -> TestSupportResult<(ShapeId, ShapeId, Vec<ShapeId>)> {
    let (a, b) = require_drawn_shape_ids(doc)?;
    let expected_ids = require_sorted_drawn_shape_ids(doc)?;
    let (lower, higher) = ordered_pair(doc, a, b, "after drawing")?;
    assert_relative_order(doc, lower, higher, "after drawing")?;
    Ok((lower, higher, expected_ids))
}

fn click_and_verify_topmost(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    click_point: gpui::Point<gpui::Pixels>,
    expected_shape: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(click_point, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(click_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let selection = read_selection(visual_cx, view);
    let selected = selected_shape_id(&selection)
        .ok_or_else(|| TestSupportError::missing("selection", "after clicking"))?;
    if selected != expected_shape {
        return Err(TestSupportError::expectation(format!(
            "expected overlapping click to select the top-most shape; got {selected:?}"
        )));
    }
    Ok(())
}
