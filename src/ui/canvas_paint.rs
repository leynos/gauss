//! Canvas painting helpers for Phase 0.
//!
//! Phase 0 uses GPUI’s `Canvas` element as a low-level drawing surface. This
//! module bridges the pure `crate::model` document types to GPUI primitives
//! such as `PathBuilder` and `Window::paint_path`.

#![expect(
    clippy::float_arithmetic,
    reason = "canvas painting relies on floating-point layout and geometry"
)]

use gpui::{
    App, Bounds, Path, PathBuilder, Pixels, Styled as _, Window, canvas, fill, point, px, rgba,
};

use crate::model::{Document, Rgba as ModelRgba, SegmentKind, Selection, Shape, Viewport};
use crate::ui::selection_overlays::{OverlayMarker, SelectionOverlays, compute_selection_overlays};
use crate::ui::selection_utils::selected_shape_ids;

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
    let overlays = compute_selection_overlays(doc, selection, viewport);
    paint_bbox_overlays(window, &overlays, DashPattern::new(6.0, 4.0));
    paint_handle_overlays(window, &overlays);
    paint_anchor_markers(window, &overlays);

    // Draw selection outlines last so the selection colour stays visible.
    for shape_id in selected_shape_ids(selection) {
        let Some(shape) = doc.shapes.iter().find(|shape| shape.id == shape_id) else {
            continue;
        };

        let (_, stroke_path) = build_paths(shape, viewport);
        let Some(path) = stroke_path else {
            continue;
        };

        window.paint_path(path, rgba(0x1d4e_d8ff));
    }
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
            PathBuilder::stroke(px(shape.style.stroke_width * viewport.zoom())),
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
                    point(px(end_pos.x), px(end_pos.y)),
                    point(px(c1_screen.x), px(c1_screen.y)),
                    point(px(c2_screen.x), px(c2_screen.y)),
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
                point(px(end_pos.x), px(end_pos.y)),
                point(px(c1_screen.x), px(c1_screen.y)),
                point(px(c2_screen.x), px(c2_screen.y)),
            );
        }

        builder.close();
    }

    builder.build().ok()
}

fn paint_bbox_overlays(window: &mut Window, overlays: &SelectionOverlays, pattern: DashPattern) {
    let mut builder = PathBuilder::stroke(px(1.0));

    for edge in &overlays.bbox_edges {
        add_dashed_line(&mut builder, edge.start, edge.end, pattern);
    }

    let Some(path) = builder.build().ok() else {
        return;
    };

    window.paint_path(path, rgba(0xb0b0_b080));
}

fn paint_handle_overlays(window: &mut Window, overlays: &SelectionOverlays) {
    let mut builder = PathBuilder::stroke(px(1.0));

    for line in &overlays.handle_lines {
        builder.move_to(point(px(line.start.x), px(line.start.y)));
        builder.line_to(point(px(line.end.x), px(line.end.y)));
    }

    let Some(path) = builder.build().ok() else {
        return;
    };

    window.paint_path(path, rgba(0x6b72_80b0));

    for marker in &overlays.handle_markers {
        paint_square_marker(window, marker, 0x6b72_80ff);
    }
}

fn paint_anchor_markers(window: &mut Window, overlays: &SelectionOverlays) {
    for marker in &overlays.anchor_markers {
        paint_square_marker(window, marker, 0x1d4e_d8ff);
    }
}

fn paint_square_marker(window: &mut Window, marker: &OverlayMarker, colour: u32) {
    let half = marker.size / 2.0;
    let min_x = marker.centre.x - half;
    let min_y = marker.centre.y - half;
    let max_x = marker.centre.x + half;
    let max_y = marker.centre.y + half;

    let mut builder = PathBuilder::fill();
    builder.move_to(point(px(min_x), px(min_y)));
    builder.line_to(point(px(max_x), px(min_y)));
    builder.line_to(point(px(max_x), px(max_y)));
    builder.line_to(point(px(min_x), px(max_y)));
    builder.close();

    let Some(path) = builder.build().ok() else {
        return;
    };

    window.paint_path(path, rgba(colour));
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

fn add_dashed_line(
    builder: &mut PathBuilder,
    start: crate::model::Vec2,
    end: crate::model::Vec2,
    pattern: DashPattern,
) {
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
