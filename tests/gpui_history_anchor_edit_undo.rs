//! GPUI headless integration tests for Phase 0 anchor insertion/deletion.

mod common;

use common::{
    anchor_to_canvas_point, assert_vec2_close, canvas_bounds, click_canvas_and_wait,
    ensure_initial_draw, init_test_app, read_document, read_history_len, require_draw_shape,
    simulate_escape, simulate_key,
};
use gauss::model::{SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult, math};

#[derive(Clone, Copy, Debug)]
struct PathCounts {
    anchors: usize,
    segments: usize,
}

#[derive(Clone, Copy, Debug)]
struct AnchorSelection {
    shape_id: ShapeId,
    anchor: usize,
}

#[derive(Clone, Copy, Debug)]
struct DrawnPathSetup {
    bounds: gpui::Bounds<gpui::Pixels>,
    p1: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
    start_pos: Vec2,
    end_pos: Vec2,
}

fn select_segment0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    select_point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let expected_segment = SelItem::Segment {
        shape: shape_id,
        seg: 0,
    };
    if !selection.contains(&expected_segment) {
        return Err(TestSupportError::expectation(format!(
            "expected segment selection; selection={selection:?}"
        )));
    }

    visual_cx.simulate_mouse_up(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

fn require_anchor_pos(shape: &Shape, index: usize, context: &str) -> TestSupportResult<Vec2> {
    shape
        .path
        .anchors
        .get(index)
        .map(|anchor| anchor.pos)
        .ok_or_else(|| TestSupportError::missing(format!("anchor {index}"), context))
}

fn require_anchor_len(shape: &Shape, expected: usize, context: &str) -> TestSupportResult<()> {
    if shape.path.anchors.len() != expected {
        return Err(TestSupportError::expectation(format!(
            "{context}: expected {expected} anchors, got {}",
            shape.path.anchors.len()
        )));
    }
    Ok(())
}

fn require_segment_len(shape: &Shape, expected: usize, context: &str) -> TestSupportResult<()> {
    if shape.path.segments.len() != expected {
        return Err(TestSupportError::expectation(format!(
            "{context}: expected {expected} segments, got {}",
            shape.path.segments.len()
        )));
    }
    Ok(())
}

fn require_path_counts(
    shape: &Shape,
    anchors: usize,
    segments: usize,
    context: &str,
) -> TestSupportResult<()> {
    require_anchor_len(shape, anchors, context)?;
    require_segment_len(shape, segments, context)?;
    Ok(())
}

fn read_draw_shape(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> TestSupportResult<Shape> {
    let doc = read_document(visual_cx, view);
    Ok(require_draw_shape(&doc, context)?.clone())
}

fn read_shape_with_counts(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    counts: PathCounts,
    context: &str,
) -> TestSupportResult<Shape> {
    let shape = read_draw_shape(visual_cx, view, context)?;
    require_path_counts(&shape, counts.anchors, counts.segments, context)?;
    Ok(shape)
}

fn require_anchor_pair(shape: &Shape, context: &str) -> TestSupportResult<(Vec2, Vec2)> {
    let start_pos = require_anchor_pos(shape, 0, context)?;
    let end_pos = require_anchor_pos(shape, 1, context)?;
    Ok((start_pos, end_pos))
}

fn expect_selection_contains_anchor(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    anchor_selection: AnchorSelection,
    context: &str,
) -> TestSupportResult<()> {
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.contains(&SelItem::Anchor {
        shape: anchor_selection.shape_id,
        anchor: anchor_selection.anchor,
    }) {
        return Err(TestSupportError::expectation(format!(
            "expected anchor selection ({context}); selection={selection:?}"
        )));
    }
    Ok(())
}

fn expect_selection_empty(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> TestSupportResult<()> {
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.items.is_empty() {
        return Err(TestSupportError::expectation(format!(
            "expected selection to be empty ({context}); selection={selection:?}"
        )));
    }
    Ok(())
}

fn draw_two_point_path(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<DrawnPathSetup> {
    let bounds = canvas_bounds(visual_cx)?;
    let p1 = point(
        bounds.origin.x + px(common::CANVAS_PADDING_PX),
        bounds.origin.y + px(common::CANVAS_PADDING_PX),
    );
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(common::CANVAS_PADDING_PX),
        bounds.origin.y + bounds.size.height - px(common::CANVAS_PADDING_PX),
    );
    click_canvas_and_wait(visual_cx, p1);
    click_canvas_and_wait(visual_cx, p2);

    let shape_before = read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 2,
            segments: 1,
        },
        "after drawing",
    )?;
    let (start_pos, end_pos) = require_anchor_pair(&shape_before, "after drawing")?;

    Ok(DrawnPathSetup {
        bounds,
        p1,
        shape_id: shape_before.id,
        start_pos,
        end_pos,
    })
}

fn insert_anchor_at_midpoint(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    setup: &DrawnPathSetup,
    midpoint: Vec2,
) -> TestSupportResult<()> {
    let select_point = anchor_to_canvas_point(&setup.bounds, midpoint, setup.p1);
    select_segment0(visual_cx, view, select_point, setup.shape_id)?;

    simulate_key(visual_cx, "i", Modifiers::none());
    let shape_after_insert = read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 3,
            segments: 2,
        },
        "after insert",
    )?;

    let inserted_pos = require_anchor_pos(&shape_after_insert, 1, "after insert")?;
    assert_vec2_close(
        inserted_pos,
        midpoint,
        "inserted anchor should be at midpoint",
    )?;
    expect_selection_contains_anchor(
        visual_cx,
        view,
        AnchorSelection {
            shape_id: setup.shape_id,
            anchor: 1,
        },
        "after insert",
    )?;

    Ok(())
}

fn delete_selected_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<()> {
    simulate_key(visual_cx, "backspace", Modifiers::none());
    read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 2,
            segments: 1,
        },
        "after delete",
    )?;
    expect_selection_empty(visual_cx, view, "after delete")?;
    Ok(())
}

fn assert_undo_redo_round_trip(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<()> {
    common::simulate_document_undo(visual_cx);
    read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 3,
            segments: 2,
        },
        "after undo delete",
    )?;

    common::simulate_document_undo(visual_cx);
    read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 2,
            segments: 1,
        },
        "after undo insert",
    )?;

    common::simulate_document_redo(visual_cx);
    read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 3,
            segments: 2,
        },
        "after redo insert",
    )?;

    common::simulate_document_redo(visual_cx);
    read_shape_with_counts(
        visual_cx,
        view,
        PathCounts {
            anchors: 2,
            segments: 1,
        },
        "after redo delete",
    )?;
    Ok(())
}

#[gpui::test]
fn insert_and_delete_anchor_are_doc_undoable(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let setup = draw_two_point_path(visual_cx, &view).expect("expected base draw path");
    simulate_escape(visual_cx);

    let midpoint = Vec2::new(
        math::midpoint(setup.start_pos.x, setup.end_pos.x),
        math::midpoint(setup.start_pos.y, setup.end_pos.y),
    );
    let len_before_insert = read_history_len(visual_cx, &view);

    insert_anchor_at_midpoint(visual_cx, &view, &setup, midpoint)
        .expect("expected anchor insertion to succeed");
    assert_eq!(
        read_history_len(visual_cx, &view),
        len_before_insert + 1,
        "expected one undo entry for anchor insertion"
    );

    delete_selected_anchor(visual_cx, &view)
        .expect("expected delete to remove the inserted anchor");
    assert_eq!(
        read_history_len(visual_cx, &view),
        len_before_insert + 2,
        "expected two undo entries after insert + delete"
    );
    assert_undo_redo_round_trip(visual_cx, &view)
        .expect("expected undo/redo to round-trip anchor edits");
}
