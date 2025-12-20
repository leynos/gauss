//! Embedded SVG icons for the Gauss chrome UI.

use std::sync::{Arc, LazyLock};

use gpui::{Image, ImageFormat, IntoElement, Styled as _, img, px};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiIcon {
    AlignHorizontalCenter,
    AlignHorizontalLeft,
    AlignHorizontalRight,
    AlignVerticalBottom,
    AlignVerticalCenter,
    AlignVerticalTop,
    DrawCircle,
    DrawCurve,
    DrawPath,
    DrawSquare,
    EditRedo,
    EditUndo,
    FileNew,
    FileOpen,
    FileSave,
    Select,
    Settings,
    SnapToGrid,
    WindowClose,
    WindowMaximize,
    WindowMinimize,
    ZoomArea,
    ZoomIn,
    ZoomOut,
}

pub(crate) fn icon_element(icon: UiIcon, size: f32) -> impl IntoElement {
    let image = icon_image(icon);
    img(image).size(px(size))
}

fn icon_image(icon: UiIcon) -> Arc<Image> {
    match icon {
        UiIcon::AlignHorizontalCenter => ALIGN_HORIZONTAL_CENTER.clone(),
        UiIcon::AlignHorizontalLeft => ALIGN_HORIZONTAL_LEFT.clone(),
        UiIcon::AlignHorizontalRight => ALIGN_HORIZONTAL_RIGHT.clone(),
        UiIcon::AlignVerticalBottom => ALIGN_VERTICAL_BOTTOM.clone(),
        UiIcon::AlignVerticalCenter => ALIGN_VERTICAL_CENTER.clone(),
        UiIcon::AlignVerticalTop => ALIGN_VERTICAL_TOP.clone(),
        UiIcon::DrawCircle => DRAW_CIRCLE.clone(),
        UiIcon::DrawCurve => DRAW_CURVE.clone(),
        UiIcon::DrawPath => DRAW_PATH.clone(),
        UiIcon::DrawSquare => DRAW_SQUARE.clone(),
        UiIcon::EditRedo => EDIT_REDO.clone(),
        UiIcon::EditUndo => EDIT_UNDO.clone(),
        UiIcon::FileNew => FILE_NEW.clone(),
        UiIcon::FileOpen => FILE_OPEN.clone(),
        UiIcon::FileSave => FILE_SAVE.clone(),
        UiIcon::Select => SELECT.clone(),
        UiIcon::Settings => SETTINGS.clone(),
        UiIcon::SnapToGrid => SNAP_TO_GRID.clone(),
        UiIcon::WindowClose => WINDOW_CLOSE.clone(),
        UiIcon::WindowMaximize => WINDOW_MAXIMIZE.clone(),
        UiIcon::WindowMinimize => WINDOW_MINIMIZE.clone(),
        UiIcon::ZoomArea => ZOOM_AREA.clone(),
        UiIcon::ZoomIn => ZOOM_IN.clone(),
        UiIcon::ZoomOut => ZOOM_OUT.clone(),
    }
}

fn svg_image(bytes: &'static [u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Svg, bytes.to_vec()))
}

static ALIGN_HORIZONTAL_CENTER: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/align-horizontal-center.svg"
    )))
});
static ALIGN_HORIZONTAL_LEFT: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/align-horizontal-left.svg"
    )))
});
static ALIGN_HORIZONTAL_RIGHT: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/align-horizontal-right.svg"
    )))
});
static ALIGN_VERTICAL_BOTTOM: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/align-vertical-bottom.svg"
    )))
});
static ALIGN_VERTICAL_CENTER: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/align-vertical-center.svg"
    )))
});
static ALIGN_VERTICAL_TOP: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/align-vertical-top.svg"
    )))
});
static DRAW_CIRCLE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/draw-circle.svg"
    )))
});
static DRAW_CURVE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/draw-curve.svg"
    )))
});
static DRAW_PATH: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/draw-path.svg"
    )))
});
static DRAW_SQUARE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/draw-square.svg"
    )))
});
static EDIT_REDO: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/edit-redo.svg"
    )))
});
static EDIT_UNDO: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/edit-undo.svg"
    )))
});
static FILE_NEW: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/file-new.svg"
    )))
});
static FILE_OPEN: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/file-open.svg"
    )))
});
static FILE_SAVE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/file-save.svg"
    )))
});
static SELECT: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/select.svg"
    )))
});
static SETTINGS: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/settings.svg"
    )))
});
static SNAP_TO_GRID: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/snap-to-grid.svg"
    )))
});
static WINDOW_CLOSE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/window-close.svg"
    )))
});
static WINDOW_MAXIMIZE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/window-maximize.svg"
    )))
});
static WINDOW_MINIMIZE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/window-minimize.svg"
    )))
});
static ZOOM_AREA: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/zoom-area.svg"
    )))
});
static ZOOM_IN: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/zoom-in.svg"
    )))
});
static ZOOM_OUT: LazyLock<Arc<Image>> = LazyLock::new(|| {
    svg_image(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/icons/zoom-out.svg"
    )))
});
