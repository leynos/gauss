//! GPUI headless integration tests for Phase 0 draw-mode Bézier auto mode.
//!
//! Phase 0 supports two draw edge modes:
//!
//! - `Line`: new segments are straight lines with no handles.
//! - `Bezier (auto)`: new segments are cubic Béziers and the editor synthesises
//!   control points using a Catmull–Rom-to-cubic conversion.
//!
//! This test asserts that switching to Bézier auto via `Tab` yields cubic
//! segments with the expected handle positions.

mod common;

use common::{
    assert_vec2_close, draw_point, ensure_initial_draw, init_test_app, require_draw_shape,
};
use gauss::model::{SegmentKind, Shape, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, point, px};

const CATMULL_ROM_TENSION: f32 = 1.0;

fn catmull_rom_controls(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> (Vec2, Vec2) {
    let t = CATMULL_ROM_TENSION / 6.0;
    let c1 = p1.add(p2.sub(p0).mul(t));
    let c2 = p2.sub(p3.sub(p1).mul(t));
    (c1, c2)
}

fn assert_segment_kind_is_cubic(shape: &Shape) {
    assert!(
        shape
            .path
            .segments
            .iter()
            .all(|kind| *kind == SegmentKind::Cubic),
        "expected all segments to be cubic in Bézier auto mode; got segments={:?}",
        shape.path.segments
    );
}

fn require_anchor_pos(shape: &Shape, index: usize, context: &str) -> Vec2 {
    let Some(anchor) = shape.path.anchors.get(index) else {
        panic!("expected anchor {index} to exist: {context}");
    };
    anchor.pos
}

fn require_handle_out(shape: &Shape, anchor_index: usize, context: &str) -> Vec2 {
    let Some(anchor) = shape.path.anchors.get(anchor_index) else {
        panic!("expected anchor {anchor_index} to exist: {context}");
    };
    let Some(handle) = anchor.handle_out else {
        panic!("expected handle_out on anchor {anchor_index}: {context}");
    };
    handle
}

fn require_handle_in(shape: &Shape, anchor_index: usize, context: &str) -> Vec2 {
    let Some(anchor) = shape.path.anchors.get(anchor_index) else {
        panic!("expected anchor {anchor_index} to exist: {context}");
    };
    let Some(handle) = anchor.handle_in else {
        panic!("expected handle_in on anchor {anchor_index}: {context}");
    };
    handle
}

#[gpui::test]
fn tab_switches_to_bezier_auto_and_synthesises_handles(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + px(12.0),
    );
    let p3 = point(
        bounds.origin.x + bounds.size.width - px(12.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    let p4 = point(
        bounds.origin.x + px(12.0),
        bounds.origin.y + bounds.size.height - px(12.0),
    );

    draw_point(visual_cx, p1);
    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();

    draw_point(visual_cx, p2);
    draw_point(visual_cx, p3);
    draw_point(visual_cx, p4);

    let doc = visual_cx.read(|app| view.read(app).document().clone());
    let shape = require_draw_shape(&doc, "after drawing bezier auto points");
    assert_eq!(
        shape.path.anchors.len(),
        4,
        "expected four anchors after drawing"
    );
    assert_eq!(
        shape.path.segments.len(),
        3,
        "expected three segments after drawing"
    );
    assert_segment_kind_is_cubic(shape);

    let a0 = require_anchor_pos(shape, 0, "anchor0");
    let a1 = require_anchor_pos(shape, 1, "anchor1");
    let a2 = require_anchor_pos(shape, 2, "anchor2");
    let a3 = require_anchor_pos(shape, 3, "anchor3");

    // Segment 1 (a1->a2) is updated when a3 is appended, so it uses the full
    // four-point window [a0, a1, a2, a3].
    let (expected_c1, expected_c2) = catmull_rom_controls(a0, a1, a2, a3);
    let actual_c1 = require_handle_out(shape, 1, "segment1 start handle_out");
    let actual_c2 = require_handle_in(shape, 2, "segment1 end handle_in");

    assert_vec2_close(actual_c1, expected_c1, "segment1 handle_out");
    assert_vec2_close(actual_c2, expected_c2, "segment1 handle_in");
}
