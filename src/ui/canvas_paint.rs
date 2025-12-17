//! Canvas painting helpers for Phase 0.
//!
//! Phase 0 uses GPUI’s `Canvas` element as a low-level drawing surface. This
//! module bridges the pure `crate::model` document types to GPUI primitives
//! such as `PathBuilder` and `Window::paint_path`.

use gpui::{
    App, Bounds, Path, PathBuilder, Pixels, Styled as _, Window, canvas, fill, point, px, rgba,
};

use crate::model::{Document, Rgba as ModelRgba, SegmentKind, SelItem, Selection, Shape, Viewport};

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
                    point(px(end_pos.x), px(end_pos.y)),
                    point(px(c1_screen.x), px(c1_screen.y)),
                    point(px(c2_screen.x), px(c2_screen.y)),
                );
            }
        }
    }

    if shape.path.closed {
        builder.close();
    }

    builder.build().ok()
}

fn model_rgba_to_hex(color: ModelRgba) -> u32 {
    (u32::from(color.r) << 24)
        | (u32::from(color.g) << 16)
        | (u32::from(color.b) << 8)
        | u32::from(color.a)
}
