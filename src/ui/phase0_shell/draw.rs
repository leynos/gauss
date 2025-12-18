//! Draw-mode behaviour and document history wiring for the Phase 0 shell.
//!
//! Phase 0 intentionally keeps draw mode simple:
//!
//! - Click appends anchors to an open path.
//! - Clicking near the first anchor closes the path.
//! - Each click is one undo step.
//! - When in "Bezier (auto)" mode, we synthesise cubic handles using a
//!   Catmull–Rom-to-cubic conversion.

use gpui_component::history::HistoryItem;

use crate::model::{
    Anchor, DocChange, DocOp, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2,
};

use super::Phase0Shell;

const SNAP_RADIUS_PX: f32 = 10.0;
const CATMULL_ROM_TENSION: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ToolMode {
    Draw,
    Manipulate,
}

impl ToolMode {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Manipulate => "Manipulate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DrawEdgeMode {
    Line,
    BezierAuto,
}

impl DrawEdgeMode {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::BezierAuto => "Bezier (auto)",
        }
    }

    pub(super) const fn toggle(self) -> Self {
        match self {
            Self::Line => Self::BezierAuto,
            Self::BezierAuto => Self::Line,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct DocHistoryItem {
    version: usize,
    pub(super) change: DocChange,
}

impl PartialEq for DocHistoryItem {
    fn eq(&self, other: &Self) -> bool {
        self.change == other.change
    }
}

impl HistoryItem for DocHistoryItem {
    fn version(&self) -> usize {
        self.version
    }

    fn set_version(&mut self, version: usize) {
        self.version = version;
    }
}

impl DocHistoryItem {
    pub(super) const fn new(change: DocChange) -> Self {
        Self { version: 0, change }
    }
}

impl Phase0Shell {
    pub(super) fn handle_canvas_click(&mut self, position: gpui::Point<gpui::Pixels>) -> bool {
        let cursor_screen = Vec2::new(f32::from(position.x), f32::from(position.y));
        self.last_canvas_click_screen = Some(cursor_screen);

        if self.tool_mode != ToolMode::Draw {
            return false;
        }

        let cursor_world = self.viewport.screen_to_world(cursor_screen);
        self.draw_click_world(cursor_world)
    }

    fn draw_click_world(&mut self, cursor_world: Vec2) -> bool {
        let Some(active) = self.draw_active_shape else {
            let shape = new_open_shape(cursor_world, self.current_style.clone());
            let index = self.document.shapes.len();
            self.apply_doc_change(DocChange {
                ops: vec![DocOp::InsertShape {
                    index,
                    shape: shape.clone(),
                }],
            });

            self.draw_active_shape = Some(shape.id);
            return true;
        };

        let Some(index) = self.document.find_index(active) else {
            self.draw_active_shape = None;
            return self.draw_click_world(cursor_world);
        };

        let Some(existing) = self.document.shapes.get(index).cloned() else {
            self.draw_active_shape = None;
            return false;
        };

        if should_close_path(&existing.path, cursor_world, self.viewport.zoom) {
            let closed = close_shape(existing.clone());
            self.apply_doc_change(replace_shape_change(index, existing, closed));
            self.tool_mode = ToolMode::Manipulate;
            self.draw_active_shape = None;
            return true;
        }

        let appended = append_anchor(existing.clone(), cursor_world, self.edge_mode);
        self.apply_doc_change(replace_shape_change(index, existing, appended));
        true
    }

    pub(super) fn apply_doc_change(&mut self, change: DocChange) {
        change.apply(&mut self.document);
        self.document_history.push(DocHistoryItem::new(change));
    }

    pub(super) fn undo_document(&mut self) {
        let Some(group) = self.document_history.undo() else {
            return;
        };

        for item in group {
            item.change.apply_inverse(&mut self.document);
        }
    }

    pub(super) fn redo_document(&mut self) {
        let Some(group) = self.document_history.redo() else {
            return;
        };

        for item in group {
            item.change.apply(&mut self.document);
        }
    }
}

fn new_open_shape(first_anchor: Vec2, style: PaintStyle) -> Shape {
    Shape {
        id: ShapeId::new_v4(),
        z: 0,
        style,
        path: PathGeom {
            anchors: vec![Anchor::new(first_anchor)],
            segments: Vec::new(),
            closed: false,
        },
    }
}

const fn segment_kind_for_edge_mode(edge_mode: DrawEdgeMode) -> SegmentKind {
    match edge_mode {
        DrawEdgeMode::Line => SegmentKind::Line,
        DrawEdgeMode::BezierAuto => SegmentKind::Cubic,
    }
}

fn append_anchor(mut shape: Shape, pos: Vec2, edge_mode: DrawEdgeMode) -> Shape {
    let new_anchor_index = shape.path.anchors.len();
    shape.path.anchors.push(Anchor::new(pos));
    shape
        .path
        .segments
        .push(segment_kind_for_edge_mode(edge_mode));

    match edge_mode {
        DrawEdgeMode::Line => {
            clear_line_segment_handles(&mut shape.path, new_anchor_index);
        }
        DrawEdgeMode::BezierAuto => {
            update_catmull_rom_handles(&mut shape.path, new_anchor_index);
        }
    }

    shape
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

const fn close_shape(mut shape: Shape) -> Shape {
    shape.path.closed = true;
    if shape.style.fill.is_none() {
        shape.style.fill = Some(Rgba::new(0, 0, 0, 32));
    }
    shape
}

fn replace_shape_change(index: usize, from: Shape, to: Shape) -> DocChange {
    DocChange {
        ops: vec![
            DocOp::RemoveShape { index, shape: from },
            DocOp::InsertShape { index, shape: to },
        ],
    }
}

fn should_close_path(path: &PathGeom, cursor_world: Vec2, zoom: f32) -> bool {
    let Some(first) = path.anchors.first() else {
        return false;
    };

    if path.anchors.len() < 3 {
        return false;
    }

    let snap_world = SNAP_RADIUS_PX / zoom;
    first.pos.distance(cursor_world) <= snap_world
}
