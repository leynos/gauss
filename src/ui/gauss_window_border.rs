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
    AnyElement, AnyView, App, Bounds, Context, CursorStyle, Decorations, Div, Entity,
    HitboxBehavior, Hsla, InteractiveElement as _, IntoElement, MouseButton, ParentElement, Pixels,
    Point, Render, RenderOnce, ResizeEdge, Size, Stateful, Styled as _, Tiling, Window, canvas,
    div, point, prelude::FluentBuilder as _, px,
};

use gpui_component::ActiveTheme;

#[cfg(not(target_os = "linux"))]
const SHADOW_SIZE: Pixels = px(0.0);
#[cfg(target_os = "linux")]
const SHADOW_SIZE: Pixels = px(12.0);

const BORDER_SIZE: Pixels = px(1.0);
const BORDER_RADIUS: Pixels = px(0.0);

/// Create a new Gauss window border with maximized-aware resize zones.
pub const fn gauss_window_border() -> GaussWindowBorder {
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
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Create the resize detection canvas element.
    ///
    /// This canvas inserts a hitbox covering the window and updates the cursor
    /// style based on which resize edge the mouse is over.
    fn create_resize_canvas() -> impl IntoElement {
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
                if window.is_maximized() {
                    return;
                }

                let mouse = window.mouse_position();
                let size = window.window_bounds().get_bounds().size;
                if let Some(edge) = resize_edge(mouse, SHADOW_SIZE, size) {
                    window.set_cursor_style(cursor_style_for_edge(edge), &hitbox);
                }
            },
        )
        .size_full()
        .absolute()
    }

    /// Apply tiling-aware padding and corner rounding to the outer div.
    fn apply_tiling_styles(div: Stateful<Div>, tiling: Tiling) -> Stateful<Div> {
        div.when(!(tiling.top || tiling.right), |d| {
            d.rounded_tr(BORDER_RADIUS)
        })
        .when(!(tiling.top || tiling.left), |d| {
            d.rounded_tl(BORDER_RADIUS)
        })
        .when(!tiling.top, |d| d.pt(SHADOW_SIZE))
        .when(!tiling.bottom, |d| d.pb(SHADOW_SIZE))
        .when(!tiling.left, |d| d.pl(SHADOW_SIZE))
        .when(!tiling.right, |d| d.pr(SHADOW_SIZE))
    }

    /// Apply resize mouse-down handler when not maximized.
    fn apply_resize_handler(div: Stateful<Div>, is_maximized: bool) -> Stateful<Div> {
        div.when(!is_maximized, |d| {
            d.on_mouse_down(MouseButton::Left, handle_resize_mouse_down)
        })
    }

    /// Render the inner content border with styling based on tiling state.
    fn render_inner_border(self, tiling: Tiling, app: &App) -> Div {
        // Inline corner rounding (same logic as apply_tiling_styles but for Div)
        let inner_div = div()
            .when(!(tiling.top || tiling.right), |d| {
                d.rounded_tr(BORDER_RADIUS)
            })
            .when(!(tiling.top || tiling.left), |d| {
                d.rounded_tl(BORDER_RADIUS)
            });

        apply_border_styling(inner_div, tiling, app)
            .on_mouse_move(|_e, _, ctx| {
                ctx.stop_propagation();
            })
            .bg(app.theme().background)
            .size_full()
            .children(self.children)
    }
}

impl ParentElement for GaussWindowBorder {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for GaussWindowBorder {
    fn render(self, window: &mut Window, app: &mut App) -> impl IntoElement {
        let decorations = window.window_decorations();
        let is_maximized = window.is_maximized();
        window.set_client_inset(SHADOW_SIZE);

        let styled_backdrop = match decorations {
            Decorations::Server => div()
                .id("gauss-window-backdrop")
                .bg(gpui::transparent_black()),
            Decorations::Client { tiling, .. } => {
                let mut outer = div()
                    .id("gauss-window-backdrop")
                    .bg(gpui::transparent_black());

                // Only add resize hit detection when NOT maximized.
                if !is_maximized {
                    outer = outer.child(Self::create_resize_canvas());
                }

                let styled_outer = Self::apply_tiling_styles(outer, tiling);
                Self::apply_resize_handler(styled_outer, is_maximized)
            }
        };

        let inner = match decorations {
            Decorations::Server => div()
                .on_mouse_move(|_e, _, ctx| ctx.stop_propagation())
                .bg(app.theme().background)
                .size_full()
                .children(self.children),
            Decorations::Client { tiling } => self.render_inner_border(tiling, app),
        };

        styled_backdrop.size_full().child(inner)
    }
}

/// Handle mouse-down event for window resize.
///
/// This is extracted as a standalone function to reduce nesting depth
/// in `apply_resize_handler`.
fn handle_resize_mouse_down(_: &gpui::MouseDownEvent, window: &mut Window, _: &mut App) {
    if window.is_maximized() {
        return;
    }
    let size = window.window_bounds().get_bounds().size;
    let pos = window.mouse_position();

    if let Some(edge) = resize_edge(pos, SHADOW_SIZE, size) {
        window.start_window_resize(edge);
    }
}

/// Map a resize edge to the appropriate cursor style.
const fn cursor_style_for_edge(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

/// Apply border styling including edges and drop shadow.
fn apply_border_styling(div: Div, tiling: Tiling, cx: &App) -> Div {
    div.border_color(cx.theme().window_border)
        .when(!tiling.top, |d| d.border_t(BORDER_SIZE))
        .when(!tiling.bottom, |d| d.border_b(BORDER_SIZE))
        .when(!tiling.left, |d| d.border_l(BORDER_SIZE))
        .when(!tiling.right, |d| d.border_r(BORDER_SIZE))
        .when(!tiling.is_tiled(), |d| {
            d.shadow(vec![gpui::BoxShadow {
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
        })
}

/// Determine which resize edge (if any) the mouse position is over.
///
/// Corners take precedence at intersections (e.g., top-left wins over top).
/// Ported from `gpui-component::window_border` with identical logic.
fn resize_edge(pos: Point<Pixels>, shadow_size: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let in_top = pos.y < shadow_size;
    let in_bottom = pos.y > size.height - shadow_size;
    let in_left = pos.x < shadow_size;
    let in_right = pos.x > size.width - shadow_size;

    // Check corners first (they take precedence at intersections)
    match (in_top, in_bottom, in_left, in_right) {
        (true, _, true, _) => return Some(ResizeEdge::TopLeft),
        (true, _, _, true) => return Some(ResizeEdge::TopRight),
        (_, true, true, _) => return Some(ResizeEdge::BottomLeft),
        (_, true, _, true) => return Some(ResizeEdge::BottomRight),
        _ => {}
    }

    // Check edges
    if in_top {
        Some(ResizeEdge::Top)
    } else if in_bottom {
        Some(ResizeEdge::Bottom)
    } else if in_left {
        Some(ResizeEdge::Left)
    } else if in_right {
        Some(ResizeEdge::Right)
    } else {
        None
    }
}

/// Window root wrapper that renders content inside our custom window border.
///
/// GPUI's `App::open_window` requires the root view to implement `Render`,
/// which means we cannot use a bare helper function. This thin wrapper
/// exists solely to satisfy that constraint while delegating all layout
/// to [`gauss_window_border`].
pub struct GaussRoot {
    view: AnyView,
}

impl GaussRoot {
    /// Wrap `view` so it can be passed to `App::open_window`.
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
