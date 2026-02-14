//! GPUI headless integration tests for Phase 0 z-order reordering.
//!
//! This test draws two identical (overlapping) shapes so that “raise” and
//! “lower” behaviour depends only on z-order, not on choosing distinct hit
//! targets.

mod common;

use common::{
    canvas_bounds, click_canvas_and_wait, demo_shape_id, ensure_initial_draw, init_test_app,
    read_document, read_history_len, read_selection, simulate_escape, simulate_key,
};
use gauss::model::{Document, SelItem, Selection, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
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

fn verify_click_selects_topmost(
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

#[expect(
    clippy::too_many_arguments,
    reason = "test helper bundles full reorder + undo verification sequence"
)]
fn verify_reorder_and_undo_sequence(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    lower: ShapeId,
    higher: ShapeId,
    expected_ids: &[ShapeId],
    len_before: usize,
) -> TestSupportResult<()> {
    simulate_key(visual_cx, "[", Modifiers::secondary_key());
    let doc_after_lower = read_document(visual_cx, view);
    let ids_after_lower = require_sorted_drawn_shape_ids(&doc_after_lower)?;
    if ids_after_lower != expected_ids {
        return Err(TestSupportError::expectation(
            "expected shape ids to remain stable after lowering",
        ));
    }
    assert_relative_order(
        &doc_after_lower,
        higher,
        lower,
        "after lowering top-most shape",
    )?;
    if read_history_len(visual_cx, view) != len_before + 1 {
        return Err(TestSupportError::expectation(
            "expected one undo entry for lower",
        ));
    }

    simulate_key(visual_cx, "]", Modifiers::secondary_key());
    let doc_after_raise = read_document(visual_cx, view);
    let ids_after_raise = require_sorted_drawn_shape_ids(&doc_after_raise)?;
    if ids_after_raise != expected_ids {
        return Err(TestSupportError::expectation(
            "expected shape ids to remain stable after raising",
        ));
    }
    assert_relative_order(&doc_after_raise, lower, higher, "after raising back to top")?;
    if read_history_len(visual_cx, view) != len_before + 2 {
        return Err(TestSupportError::expectation(
            "expected two undo entries for lower + raise",
        ));
    }

    common::simulate_document_undo(visual_cx);
    let doc_after_undo_raise = read_document(visual_cx, view);
    assert_relative_order(&doc_after_undo_raise, higher, lower, "after undoing raise")?;

    common::simulate_document_undo(visual_cx);
    let doc_after_undo_lower = read_document(visual_cx, view);
    assert_relative_order(&doc_after_undo_lower, lower, higher, "after undoing lower")
}

#[gpui::test]
fn raise_lower_reorders_overlapping_shapes_with_undo(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let points = line_points(&bounds);

    draw_overlapping_lines(visual_cx, points);

    let doc = read_document(visual_cx, &view);
    let (lower, higher, expected_ids) =
        verify_initial_shapes_and_order(&doc).expect("expected valid initial shape order");

    verify_click_selects_topmost(visual_cx, &view, points.start, higher)
        .expect("expected overlapping click to select the top-most shape");

    let len_before_reorder = read_history_len(visual_cx, &view);

    verify_reorder_and_undo_sequence(
        visual_cx,
        &view,
        lower,
        higher,
        &expected_ids,
        len_before_reorder,
    )
    .expect("expected reorder and undo sequence to be correct");
}
