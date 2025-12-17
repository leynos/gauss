//! Canvas painting helpers for Phase 0.
//!
//! Phase 0 uses GPUI’s `Canvas` element as a low-level drawing surface. This
//! module bridges the pure `crate::model` document types to GPUI primitives
//! such as `PathBuilder` and `Window::paint_path`.

use gpui::{
    App, Bounds, Path, PathBuilder, Pixels, Styled as _, Window, canvas, fill, point, px, rgba,
};

use crate::model::{Document, Rgba as ModelRgba, SegmentKind, Shape};

pub(super) fn canvas_for_document(document: &Document) -> impl gpui::IntoElement {
    let document_clone = document.clone();

    canvas(
        move |_bounds: Bounds<Pixels>, _window: &mut Window, _app: &mut App| document_clone,
        |bounds: Bounds<Pixels>, doc_to_paint: Document, window: &mut Window, _app: &mut App| {
            paint_document(bounds, &doc_to_paint, window);
        },
    )
    .flex_1()
    .border_1()
    .rounded_md()
}

fn paint_document(bounds: Bounds<Pixels>, doc: &Document, window: &mut Window) {
    window.paint_quad(fill(bounds, rgba(0xf8f8_f8ff)));

    for shape in &doc.shapes {
        paint_shape(shape, window);
    }
}

fn paint_shape(shape: &Shape, window: &mut Window) {
    let (fill_path, stroke_path) = build_paths(shape);

    if let (Some(path), Some(fill)) = (fill_path, shape.style.fill) {
        window.paint_path(path, rgba(model_rgba_to_hex(fill)));
    }

    if let (Some(path), Some(stroke)) = (stroke_path, shape.style.stroke) {
        window.paint_path(path, rgba(model_rgba_to_hex(stroke)));
    }
}

fn build_paths(shape: &Shape) -> (Option<Path<Pixels>>, Option<Path<Pixels>>) {
    let fill_path = if shape.path.closed && shape.style.fill.is_some() {
        build_path(shape, PathBuilder::fill())
    } else {
        None
    };

    let stroke_path = if shape.style.stroke.is_some() {
        build_path(shape, PathBuilder::stroke(px(shape.style.stroke_width)))
    } else {
        None
    };

    (fill_path, stroke_path)
}

fn build_path(shape: &Shape, mut builder: PathBuilder) -> Option<Path<Pixels>> {
    let first = shape.path.anchors.first()?;
    builder.move_to(point(px(first.pos.x), px(first.pos.y)));

    for (index, kind) in shape.path.segments.iter().enumerate() {
        let Some(start) = shape.path.anchors.get(index) else {
            break;
        };
        let Some(end) = shape.path.anchors.get(index + 1) else {
            break;
        };

        match kind {
            SegmentKind::Line => {
                builder.line_to(point(px(end.pos.x), px(end.pos.y)));
            }
            SegmentKind::Cubic => {
                let c1 = start.handle_out.unwrap_or(start.pos);
                let c2 = end.handle_in.unwrap_or(end.pos);
                builder.cubic_bezier_to(
                    point(px(end.pos.x), px(end.pos.y)),
                    point(px(c1.x), px(c1.y)),
                    point(px(c2.x), px(c2.y)),
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
