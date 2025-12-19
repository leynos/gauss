//! Canvas painting helpers for Phase 0.
//!
//! Phase 0 uses GPUI’s `Canvas` element as a low-level drawing surface. This
//! module bridges the pure `crate::model` document types to GPUI primitives
//! such as `PathBuilder` and `Window::paint_path`.

use gpui::{
    App, Bounds, Path, PathBuilder, Pixels, Styled as _, Window, canvas, fill, point, px, rgba,
};

use crate::model::{
    Document, Rgba as ModelRgba, SegmentKind, SelItem, Selection, Shape, Vec2, Viewport,
};

#[derive(Clone, Debug)]
struct CanvasState {
    document: Document,
    selection: Selection,
    viewport: Viewport,
}

pub(super) fn canvas_for_document(
    document: &Document,
    selection: &Selection,
    viewport: Viewport,
) -> impl gpui::IntoElement {
    let document_clone = document.clone();
    let selection_clone = selection.clone();
    let viewport_copy = viewport;

    canvas(
        move |_bounds: Bounds<Pixels>, _window: &mut Window, _app: &mut App| CanvasState {
            document: document_clone,
            selection: selection_clone,
            viewport: viewport_copy,
        },
        |bounds: Bounds<Pixels>, state: CanvasState, window: &mut Window, _app: &mut App| {
            paint_document(bounds, &state, window);
        },
    )
    .flex_1()
}

fn paint_document(bounds: Bounds<Pixels>, state: &CanvasState, window: &mut Window) {
    window.paint_quad(fill(bounds, rgba(0xf8f8_f8ff)));

    for shape in &state.document.shapes {
        paint_shape(shape, state.viewport, window);
    }

    paint_selection_overlays(&state.document, &state.selection, state.viewport, window);
}

fn paint_shape(shape: &Shape, viewport: Viewport, window: &mut Window) {
    let (fill_path, stroke_path) = build_paths(shape, viewport);

    if let (Some(path), Some(fill)) = (fill_path, shape.style.fill) {
        window.paint_path(path, rgba(model_rgba_to_hex(fill)));
    }

    if let (Some(path), Some(stroke)) = (stroke_path, shape.style.stroke) {
        window.paint_path(path, rgba(model_rgba_to_hex(stroke)));
    }
}

fn paint_selection_overlays(
    doc: &Document,
    selection: &Selection,
    viewport: Viewport,
    window: &mut Window,
) {
    for shape_id in selected_shape_ids(selection) {
        let Some(shape) = doc.shapes.iter().find(|shape| shape.id == shape_id) else {
            continue;
        };

        paint_shape_bbox(shape, viewport, window);

        let (_, stroke_path) = build_paths(shape, viewport);
        let Some(path) = stroke_path else {
            continue;
        };

        window.paint_path(path, rgba(0x1d4e_d8ff));
    }
}

fn selected_shape_ids(selection: &Selection) -> impl Iterator<Item = crate::model::ShapeId> + '_ {
    selection.items.iter().filter_map(|item| match item {
        SelItem::Shape(id) => Some(*id),
        _ => None,
    })
}

fn build_paths(shape: &Shape, viewport: Viewport) -> (Option<Path<Pixels>>, Option<Path<Pixels>>) {
    let fill_path = if shape.path.closed && shape.style.fill.is_some() {
        build_path(shape, viewport, PathBuilder::fill())
    } else {
        None
    };

    let stroke_path = if shape.style.stroke.is_some() {
        build_path(
            shape,
            viewport,
            PathBuilder::stroke(px(shape.style.stroke_width * viewport.zoom)),
        )
    } else {
        None
    };

    (fill_path, stroke_path)
}

fn build_path(shape: &Shape, viewport: Viewport, mut builder: PathBuilder) -> Option<Path<Pixels>> {
    let first = shape.path.anchors.first()?;
    let first_screen = viewport.world_to_screen(first.pos);
    builder.move_to(point(px(first_screen.x), px(first_screen.y)));

    for (index, kind) in shape.path.segments.iter().enumerate() {
        let Some(start) = shape.path.anchors.get(index) else {
            break;
        };
        let Some(end) = shape.path.anchors.get(index + 1) else {
            break;
        };

        let end_pos = viewport.world_to_screen(end.pos);
        match kind {
            SegmentKind::Line => {
                builder.line_to(point(px(end_pos.x), px(end_pos.y)));
            }
            SegmentKind::Cubic => {
                let c1 = start.handle_out.unwrap_or(start.pos);
                let c2 = end.handle_in.unwrap_or(end.pos);
                let c1_screen = viewport.world_to_screen(c1);
                let c2_screen = viewport.world_to_screen(c2);
                builder.cubic_bezier_to(
                    point(px(c1_screen.x), px(c1_screen.y)),
                    point(px(c2_screen.x), px(c2_screen.y)),
                    point(px(end_pos.x), px(end_pos.y)),
                );
            }
        }
    }

    if shape.path.closed {
        if shape.path.closing_segment == SegmentKind::Cubic
            && let (Some(last_anchor), Some(first_anchor)) =
                (shape.path.anchors.last(), shape.path.anchors.first())
        {
            let end_pos = viewport.world_to_screen(first_anchor.pos);
            let c1 = last_anchor.handle_out.unwrap_or(last_anchor.pos);
            let c2 = first_anchor.handle_in.unwrap_or(first_anchor.pos);
            let c1_screen = viewport.world_to_screen(c1);
            let c2_screen = viewport.world_to_screen(c2);

            builder.cubic_bezier_to(
                point(px(c1_screen.x), px(c1_screen.y)),
                point(px(c2_screen.x), px(c2_screen.y)),
                point(px(end_pos.x), px(end_pos.y)),
            );
        }

        builder.close();
    }

    builder.build().ok()
}

fn paint_shape_bbox(shape: &Shape, viewport: Viewport, window: &mut Window) {
    let Some((bbox_min, bbox_max)) = shape_screen_bbox(shape, viewport) else {
        return;
    };

    let padding = 3.0;
    let padded_min = Vec2::new(bbox_min.x - padding, bbox_min.y - padding);
    let padded_max = Vec2::new(bbox_max.x + padding, bbox_max.y + padding);

    let mut builder = PathBuilder::stroke(px(1.0));
    add_dashed_rect(
        &mut builder,
        padded_min,
        padded_max,
        DashPattern::new(6.0, 4.0),
    );
    let Some(path) = builder.build().ok() else {
        return;
    };

    window.paint_path(path, rgba(0xb0b0_b080));
}

fn shape_screen_bbox(shape: &Shape, viewport: Viewport) -> Option<(Vec2, Vec2)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for anchor in &shape.path.anchors {
        let screen = viewport.world_to_screen(anchor.pos);
        min_x = min_x.min(screen.x);
        min_y = min_y.min(screen.y);
        max_x = max_x.max(screen.x);
        max_y = max_y.max(screen.y);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some((Vec2::new(min_x, min_y), Vec2::new(max_x, max_y)))
}

#[derive(Clone, Copy, Debug)]
struct DashPattern {
    dash: f32,
    gap: f32,
}

impl DashPattern {
    const fn new(dash: f32, gap: f32) -> Self {
        Self { dash, gap }
    }
}

fn add_dashed_rect(builder: &mut PathBuilder, min: Vec2, max: Vec2, pattern: DashPattern) {
    let top_left = min;
    let top_right = Vec2::new(max.x, min.y);
    let bottom_right = max;
    let bottom_left = Vec2::new(min.x, max.y);

    add_dashed_line(builder, top_left, top_right, pattern);
    add_dashed_line(builder, top_right, bottom_right, pattern);
    add_dashed_line(builder, bottom_right, bottom_left, pattern);
    add_dashed_line(builder, bottom_left, top_left, pattern);
}

fn add_dashed_line(builder: &mut PathBuilder, start: Vec2, end: Vec2, pattern: DashPattern) {
    let length = start.distance(end);
    if length <= f32::EPSILON {
        return;
    }

    let direction = end.sub(start).mul(1.0 / length);

    let mut t = 0.0;
    while t < length {
        let segment_start = start.add(direction.mul(t));
        let segment_end = start.add(direction.mul((t + pattern.dash).min(length)));
        builder.move_to(point(px(segment_start.x), px(segment_start.y)));
        builder.line_to(point(px(segment_end.x), px(segment_end.y)));
        t += pattern.dash + pattern.gap;
    }
}

fn model_rgba_to_hex(color: ModelRgba) -> u32 {
    (u32::from(color.r) << 24)
        | (u32::from(color.g) << 16)
        | (u32::from(color.b) << 8)
        | u32::from(color.a)
}
