//! Geometry helpers used by the model-layer `PenTool` FSM.
//!
//! These helpers are intentionally UI-independent so draw-click transitions can
//! be tested without GPUI. They preserve the existing Phase 0 semantics for
//! path creation, Catmull-Rom handle synthesis, and close-path behaviour.

#![expect(
    clippy::float_arithmetic,
    reason = "Catmull-Rom handle synthesis is inherently floating-point"
)]

use crate::model::{
    Anchor, EdgeMode, Paint, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2,
};

const CATMULL_ROM_TENSION: f32 = 1.0;

/// Build a new open shape from the first clicked anchor.
#[must_use]
pub fn new_open_shape(id: ShapeId, first_anchor: Vec2, style: PaintStyle) -> Shape {
    Shape {
        id,
        z: 0,
        style,
        path: PathGeom {
            anchors: vec![Anchor::new(first_anchor)],
            segments: Vec::new(),
            closed: false,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    }
}

/// Append an anchor and update segment handles for the current edge mode.
#[must_use]
pub fn append_anchor(mut shape: Shape, pos: Vec2, edge_mode: EdgeMode) -> Shape {
    let new_anchor_index = shape.path.anchors.len();
    shape.path.anchors.push(Anchor::new(pos));
    shape
        .path
        .segments
        .push(segment_kind_for_edge_mode(edge_mode));

    match edge_mode {
        EdgeMode::Line => clear_line_segment_handles(&mut shape.path, new_anchor_index),
        EdgeMode::BezierAuto => update_catmull_rom_handles(&mut shape.path, new_anchor_index),
    }

    shape
}

/// Close the shape and apply closing-segment handle synthesis.
#[must_use]
pub fn close_shape(mut shape: Shape, closing_segment: SegmentKind) -> Shape {
    shape.path.closed = true;
    shape.path.closing_segment = closing_segment;
    if shape.style.fill.is_none() {
        shape.style.fill = Paint::Solid(Rgba::new(0, 0, 0, 32));
    }

    match shape.path.closing_segment {
        SegmentKind::Line => clear_closing_line_handles(&mut shape.path),
        SegmentKind::Cubic => update_closed_catmull_rom_handles(&mut shape.path),
    }

    shape
}

/// Return whether the cursor is close enough to close an open path.
#[must_use]
pub fn should_close_path(
    path: &PathGeom,
    cursor_world: Vec2,
    zoom: f32,
    snap_radius_px: f32,
) -> bool {
    let Some(first) = path.anchors.first() else {
        return false;
    };

    if path.anchors.len() < 3 {
        return false;
    }

    first.pos.distance(cursor_world) <= (snap_radius_px / zoom)
}

/// Map draw edge mode to segment kind.
#[must_use]
pub const fn segment_kind_for_edge_mode(edge_mode: EdgeMode) -> SegmentKind {
    match edge_mode {
        EdgeMode::Line => SegmentKind::Line,
        EdgeMode::BezierAuto => SegmentKind::Cubic,
    }
}

fn clear_line_segment_handles(path: &mut PathGeom, new_anchor_index: usize) {
    let Some(start_index) = new_anchor_index.checked_sub(1) else {
        return;
    };

    let Some((start, end)) = anchor_pair_mut(path, start_index) else {
        return;
    };

    start.handle_out = None;
    end.handle_in = None;
}

fn update_catmull_rom_handles(path: &mut PathGeom, new_anchor_index: usize) {
    let Some(last_seg_index) = new_anchor_index.checked_sub(1) else {
        return;
    };

    if matches!(path.segments.get(last_seg_index), Some(SegmentKind::Cubic)) {
        update_segment_catmull_rom_handles(path, last_seg_index);
    }

    let Some(prev_seg_index) = last_seg_index.checked_sub(1) else {
        return;
    };

    if matches!(path.segments.get(prev_seg_index), Some(SegmentKind::Cubic)) {
        update_segment_catmull_rom_handles(path, prev_seg_index);
    }
}

fn update_segment_catmull_rom_handles(path: &mut PathGeom, seg_index: usize) {
    let Some(p1) = path.anchors.get(seg_index).map(|a| a.pos) else {
        return;
    };
    let Some(p2) = path.anchors.get(seg_index + 1).map(|a| a.pos) else {
        return;
    };

    let p0 = seg_index
        .checked_sub(1)
        .and_then(|idx| path.anchors.get(idx).map(|a| a.pos))
        .unwrap_or(p1);
    let p3 = path.anchors.get(seg_index + 2).map_or(p2, |a| a.pos);

    let (c1, c2) = catmull_rom_controls(p0, p1, p2, p3);
    let Some((start, end)) = anchor_pair_mut(path, seg_index) else {
        return;
    };

    start.handle_out = Some(c1);
    end.handle_in = Some(c2);
}

fn catmull_rom_controls(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> (Vec2, Vec2) {
    let t = CATMULL_ROM_TENSION / 6.0;
    let c1 = p1.add(p2.sub(p0).mul(t));
    let c2 = p2.sub(p3.sub(p1).mul(t));
    (c1, c2)
}

fn anchor_pair_mut(path: &mut PathGeom, start_index: usize) -> Option<(&mut Anchor, &mut Anchor)> {
    let end_index = start_index.checked_add(1)?;
    let (head, tail) = path.anchors.split_at_mut(end_index);
    let start = head.get_mut(start_index)?;
    let end = tail.first_mut()?;
    Some((start, end))
}

fn clear_closing_line_handles(path: &mut PathGeom) {
    match path.anchors.as_mut_slice() {
        [] => {}
        [only] => {
            only.handle_in = None;
            only.handle_out = None;
        }
        many => {
            let last_index = many.len() - 1;
            let (head, tail) = many.split_at_mut(last_index);
            let (Some(first), Some(last)) = (head.first_mut(), tail.first_mut()) else {
                return;
            };

            first.handle_in = None;
            last.handle_out = None;
        }
    }
}

fn update_closed_catmull_rom_handles(path: &mut PathGeom) {
    if path.anchors.len() < 3 {
        clear_closing_line_handles(path);
        path.closing_segment = SegmentKind::Line;
        return;
    }

    if matches!(path.segments.first(), Some(SegmentKind::Cubic)) {
        update_closed_segment_catmull_rom_handles(path, 0);
    }

    if let Some(last_seg_index) = path.segments.len().checked_sub(1)
        && matches!(path.segments.get(last_seg_index), Some(SegmentKind::Cubic))
    {
        update_closed_segment_catmull_rom_handles(path, last_seg_index);
    }

    let Some(first_anchor) = path.anchors.first() else {
        return;
    };
    let Some(last_anchor) = path.anchors.last() else {
        return;
    };

    let p1 = last_anchor.pos;
    let p2 = first_anchor.pos;
    let p0 = path.anchors.iter().rev().nth(1).map_or(p1, |a| a.pos);
    let p3 = path.anchors.get(1).map_or(p2, |a| a.pos);

    let (c1, c2) = catmull_rom_controls(p0, p1, p2, p3);
    if let Some(last_anchor_mut) = path.anchors.last_mut() {
        last_anchor_mut.handle_out = Some(c1);
    }
    if let Some(first_anchor_mut) = path.anchors.first_mut() {
        first_anchor_mut.handle_in = Some(c2);
    }
}

fn update_closed_segment_catmull_rom_handles(path: &mut PathGeom, seg_index: usize) {
    let anchor_len = path.anchors.len();
    if anchor_len < 2 {
        return;
    }

    let Some(p1) = path.anchors.get(seg_index).map(|a| a.pos) else {
        return;
    };
    let Some(p2) = path.anchors.get(seg_index + 1).map(|a| a.pos) else {
        return;
    };

    let p0 = if seg_index == 0 {
        path.anchors.last().map_or(p1, |a| a.pos)
    } else {
        path.anchors.get(seg_index - 1).map_or(p1, |a| a.pos)
    };

    let p3 = if seg_index + 2 >= anchor_len {
        path.anchors.first().map_or(p2, |a| a.pos)
    } else {
        path.anchors.get(seg_index + 2).map_or(p2, |a| a.pos)
    };

    let (c1, c2) = catmull_rom_controls(p0, p1, p2, p3);
    let Some((start, end)) = anchor_pair_mut(path, seg_index) else {
        return;
    };

    start.handle_out = Some(c1);
    end.handle_in = Some(c2);
}
