//! Catmull–Rom handle synthesis for draw mode paths.
//!
//! These helpers keep the Catmull–Rom conversion logic isolated from the
//! event handling so draw mode remains easier to follow.

use crate::model::{Anchor, PathGeom, Rgba, SegmentKind, Shape, Vec2};

const CATMULL_ROM_TENSION: f32 = 1.0;

pub(super) fn clear_line_segment_handles(path: &mut PathGeom, new_anchor_index: usize) {
    let Some(start_index) = new_anchor_index.checked_sub(1) else {
        return;
    };

    let Some((start, end)) = anchor_pair_mut(path, start_index) else {
        return;
    };

    start.handle_out = None;
    end.handle_in = None;
}

pub(super) fn update_catmull_rom_handles(path: &mut PathGeom, new_anchor_index: usize) {
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

pub(super) fn close_shape(mut shape: Shape, closing_segment: SegmentKind) -> Shape {
    shape.path.closed = true;
    shape.path.closing_segment = closing_segment;
    if shape.style.fill.is_none() {
        shape.style.fill = Some(Rgba::new(0, 0, 0, 32));
    }

    match shape.path.closing_segment {
        SegmentKind::Line => clear_closing_line_handles(&mut shape.path),
        SegmentKind::Cubic => update_closed_catmull_rom_handles(&mut shape.path),
    }

    shape
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
            let last_index = many.len().saturating_sub(1);
            let (head, tail) = many.split_at_mut(last_index);
            let Some(first) = head.first_mut() else {
                return;
            };
            let Some(last) = tail.first_mut() else {
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

    // Update the first and last "internal" segments using wrap-around points
    // so the curve is continuous at the join.
    if matches!(path.segments.first(), Some(SegmentKind::Cubic)) {
        update_closed_segment_catmull_rom_handles(path, 0);
    }

    if let Some(last_seg_index) = path.segments.len().checked_sub(1)
        && matches!(path.segments.get(last_seg_index), Some(SegmentKind::Cubic))
    {
        update_closed_segment_catmull_rom_handles(path, last_seg_index);
    }

    // Closing segment from last -> first.
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
