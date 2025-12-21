//! Embedded SVG icons for the Gauss chrome UI.

use std::sync::{Arc, LazyLock};

use gpui::{Image, ImageFormat, IntoElement, Styled as _, img, px};

macro_rules! define_ui_icons {
    ($( $variant:ident => $file:literal ),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u8)]
        pub(crate) enum UiIcon {
            $( $variant ),*
        }

        impl UiIcon {
            #[cfg(test)]
            pub(crate) const ALL: [UiIcon; ICON_COUNT] = [$( UiIcon::$variant ),*];

            const fn as_index(self) -> usize {
                self as usize
            }
        }

        const ICON_COUNT: usize = 0 $(+ { let _ = stringify!($variant); 1 })*;

        static ICON_IMAGES: [LazyLock<Arc<Image>>; ICON_COUNT] = [
            $(
                LazyLock::new(|| {
                    svg_image(include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/icons/",
                        $file,
                        ".svg"
                    )))
                })
            ),*
        ];

        fn icon_image(icon: UiIcon) -> Arc<Image> {
            let Some(image) = ICON_IMAGES.get(icon.as_index()) else {
                unreachable!("UiIcon index must map to an embedded icon image");
            };
            Arc::clone(image)
        }
    };
}

define_ui_icons! {
    AlignHorizontalCenter => "align-horizontal-center",
    AlignHorizontalLeft => "align-horizontal-left",
    AlignHorizontalRight => "align-horizontal-right",
    AlignVerticalBottom => "align-vertical-bottom",
    AlignVerticalCenter => "align-vertical-center",
    AlignVerticalTop => "align-vertical-top",
    DrawCircle => "draw-circle",
    DrawCurve => "draw-curve",
    DrawPath => "draw-path",
    DrawSquare => "draw-square",
    EditRedo => "edit-redo",
    EditUndo => "edit-undo",
    FileNew => "file-new",
    FileOpen => "file-open",
    FileSave => "file-save",
    Select => "select",
    Settings => "settings",
    SnapToGrid => "snap-to-grid",
    WindowClose => "window-close",
    WindowMaximize => "window-maximize",
    WindowMinimize => "window-minimize",
    ZoomArea => "zoom-area",
    ZoomIn => "zoom-in",
    ZoomOut => "zoom-out",
}

/// Create a sized image element for a UI icon.
///
/// # Parameters
/// - `icon`: which icon to render.
/// - `size`: pixel size for the rendered image.
///
/// # Returns
/// An element implementing `IntoElement` for the sized icon image.
pub(crate) fn icon_element(icon: UiIcon, size: f32) -> impl IntoElement {
    let image = icon_image(icon);
    img(image).size(px(size))
}

fn svg_image(bytes: &'static [u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Svg, bytes.to_vec()))
}

#[cfg(test)]
mod tests {
    use gpui::{Render, TestAppContext, Window, div, prelude::*};

    use super::{ICON_COUNT, ICON_IMAGES, UiIcon, icon_element};

    struct IconTestView;

    impl Render for IconTestView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("icon-test")
                .debug_selector(|| "#icon-test".to_owned())
                .child(icon_element(UiIcon::FileOpen, 16.0))
                .child(icon_element(UiIcon::FileSave, 16.0))
        }
    }

    #[test]
    fn ui_icon_table_matches_static_images() {
        assert_eq!(UiIcon::ALL.len(), ICON_COUNT);
        assert_eq!(UiIcon::ALL.len(), ICON_IMAGES.len());
    }

    #[gpui::test]
    fn icon_element_renders_in_headless_view(cx: &mut TestAppContext) {
        cx.update(crate::ui::init);

        let (_view, visual_cx) = cx.add_window_view(|_window, _view_cx| IconTestView);
        visual_cx.update(|window, app| {
            let _draw = window.draw(app);
        });
        visual_cx.run_until_parked();

        assert!(
            visual_cx.debug_bounds("#icon-test").is_some(),
            "expected icon test view to render"
        );
    }
}
