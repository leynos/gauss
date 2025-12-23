//! Custom window border with drop shadow for Gauss.
//!
//! This is a modified version of `gpui-component`'s `window_border` that
//! respects the window's maximized state. The original implementation in
//! gpui-component does not check `is_maximized()` before setting up resize
//! zones, causing resize hit targets to block button clicks when maximized.
//!
//! Key differences from gpui-component:
//! - Resize zones are disabled when the window is maximized
//! - Cursor style changes for resize are disabled when maximized
//! - Drop shadow is still rendered (controlled by tiling state)

use gpui::{
    AnyElement, AnyView, App, Bounds, Context, CursorStyle, Decorations, Entity, HitboxBehavior,
    Hsla, InteractiveElement as _, IntoElement, MouseButton, ParentElement, Pixels, Point, Render,
    RenderOnce, ResizeEdge, Size, Styled as _, Window, canvas, div, point,
    prelude::FluentBuilder as _, px,
};

use gpui_component::ActiveTheme;

#[cfg(not(target_os = "linux"))]
const SHADOW_SIZE: Pixels = px(0.0);
#[cfg(target_os = "linux")]
const SHADOW_SIZE: Pixels = px(12.0);

const BORDER_SIZE: Pixels = px(1.0);
const BORDER_RADIUS: Pixels = px(0.0);

/// Create a new Gauss window border with maximized-aware resize zones.
pub fn gauss_window_border() -> GaussWindowBorder {
    GaussWindowBorder::new()
}

/// Window border that renders drop shadow and resize zones on Linux.
///
/// Unlike `gpui_component::window_border`, this implementation disables
/// resize zones when the window is maximized, preventing invisible hit
/// targets from blocking button clicks.
#[derive(IntoElement, Default)]
pub struct GaussWindowBorder {
    children: Vec<AnyElement>,
}

impl GaussWindowBorder {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ParentElement for GaussWindowBorder {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for GaussWindowBorder {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let decorations = window.window_decorations();
        let is_maximized = window.is_maximized();
        window.set_client_inset(SHADOW_SIZE);

        div()
            .id("gauss-window-backdrop")
            .bg(gpui::transparent_black())
            .map(|div| match decorations {
                Decorations::Server => div,
                Decorations::Client { tiling, .. } => {
                    let div = div.bg(gpui::transparent_black());

                    // Only add resize hit detection when NOT maximized.
                    // When maximized, tiling should be all true, but gpui-component
                    // doesn't respect this for the hitbox - we explicitly check.
                    let div = if !is_maximized {
                        div.child(
                            canvas(
                                |_bounds, window, _| {
                                    window.insert_hitbox(
                                        Bounds::new(
                                            point(px(0.0), px(0.0)),
                                            window.window_bounds().get_bounds().size,
                                        ),
                                        HitboxBehavior::Normal,
                                    )
                                },
                                move |_bounds, hitbox, window, _| {
                                    // Skip cursor changes if maximized (shouldn't happen
                                    // since we don't add this canvas, but defensive)
                                    if window.is_maximized() {
                                        return;
                                    }

                                    let mouse = window.mouse_position();
                                    let size = window.window_bounds().get_bounds().size;
                                    let Some(edge) = resize_edge(mouse, SHADOW_SIZE, size) else {
                                        return;
                                    };
                                    window.set_cursor_style(
                                        match edge {
                                            ResizeEdge::Top | ResizeEdge::Bottom => {
                                                CursorStyle::ResizeUpDown
                                            }
                                            ResizeEdge::Left | ResizeEdge::Right => {
                                                CursorStyle::ResizeLeftRight
                                            }
                                            ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                                                CursorStyle::ResizeUpLeftDownRight
                                            }
                                            ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                                                CursorStyle::ResizeUpRightDownLeft
                                            }
                                        },
                                        &hitbox,
                                    );
                                },
                            )
                            .size_full()
                            .absolute(),
                        )
                    } else {
                        div
                    };

                    div.when(!(tiling.top || tiling.right), |div| {
                        div.rounded_tr(BORDER_RADIUS)
                    })
                    .when(!(tiling.top || tiling.left), |div| {
                        div.rounded_tl(BORDER_RADIUS)
                    })
                    .when(!tiling.top, |div| div.pt(SHADOW_SIZE))
                    .when(!tiling.bottom, |div| div.pb(SHADOW_SIZE))
                    .when(!tiling.left, |div| div.pl(SHADOW_SIZE))
                    .when(!tiling.right, |div| div.pr(SHADOW_SIZE))
                    // Only handle resize mouse-down when NOT maximized
                    .when(!is_maximized, |div| {
                        div.on_mouse_down(MouseButton::Left, move |_, window, _| {
                            if window.is_maximized() {
                                return;
                            }
                            let size = window.window_bounds().get_bounds().size;
                            let pos = window.mouse_position();

                            if let Some(edge) = resize_edge(pos, SHADOW_SIZE, size) {
                                window.start_window_resize(edge);
                            }
                        })
                    })
                }
            })
            .size_full()
            .child(
                div()
                    .map(|div| match decorations {
                        Decorations::Server => div,
                        Decorations::Client { tiling } => div
                            .when(!(tiling.top || tiling.right), |div| {
                                div.rounded_tr(BORDER_RADIUS)
                            })
                            .when(!(tiling.top || tiling.left), |div| {
                                div.rounded_tl(BORDER_RADIUS)
                            })
                            .border_color(cx.theme().window_border)
                            .when(!tiling.top, |div| div.border_t(BORDER_SIZE))
                            .when(!tiling.bottom, |div| div.border_b(BORDER_SIZE))
                            .when(!tiling.left, |div| div.border_l(BORDER_SIZE))
                            .when(!tiling.right, |div| div.border_r(BORDER_SIZE))
                            .when(!tiling.is_tiled(), |div| {
                                div.shadow(vec![gpui::BoxShadow {
                                    color: Hsla {
                                        h: 0.,
                                        s: 0.,
                                        l: 0.,
                                        a: 0.3,
                                    },
                                    blur_radius: SHADOW_SIZE / 2.,
                                    spread_radius: px(0.),
                                    offset: point(px(0.0), px(0.0)),
                                }])
                            }),
                    })
                    .on_mouse_move(|_e, _, cx| {
                        cx.stop_propagation();
                    })
                    .bg(cx.theme().background)
                    .size_full()
                    .children(self.children),
            )
    }
}

/// Determine which resize edge (if any) the mouse position is over.
///
/// This function has high branching complexity by necessity: it must check
/// all 8 edge/corner regions in a specific order. Corners are checked first
/// (they take precedence at intersections), then edges. The order ensures
/// corners like TopLeft are detected before the general Top or Left edges.
///
/// Ported from `gpui-component::window_border` with identical logic.
fn resize_edge(pos: Point<Pixels>, shadow_size: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let edge = if pos.y < shadow_size && pos.x < shadow_size {
        ResizeEdge::TopLeft
    } else if pos.y < shadow_size && pos.x > size.width - shadow_size {
        ResizeEdge::TopRight
    } else if pos.y < shadow_size {
        ResizeEdge::Top
    } else if pos.y > size.height - shadow_size && pos.x < shadow_size {
        ResizeEdge::BottomLeft
    } else if pos.y > size.height - shadow_size && pos.x > size.width - shadow_size {
        ResizeEdge::BottomRight
    } else if pos.y > size.height - shadow_size {
        ResizeEdge::Bottom
    } else if pos.x < shadow_size {
        ResizeEdge::Left
    } else if pos.x > size.width - shadow_size {
        ResizeEdge::Right
    } else {
        return None;
    };
    Some(edge)
}

/// Window root wrapper that renders content inside our custom window border.
///
/// This serves as a replacement for `gpui_component::Root` when we need the
/// window border/shadow but don't need Sheet/Dialog/Notification features.
pub struct GaussRoot {
    view: AnyView,
}

impl GaussRoot {
    /// Create a new root wrapper for the given view entity.
    pub fn new<V: Render>(view: Entity<V>, _cx: &mut Context<Self>) -> Self {
        Self { view: view.into() }
    }
}

impl Render for GaussRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(cx.theme().font_size);
        gauss_window_border().child(self.view.clone())
    }
}
