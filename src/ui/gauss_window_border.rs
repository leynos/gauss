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

/// Apply corner rounding based on tiling state.
///
/// Corners are only rounded when they are not adjacent to a tiled edge.
/// This is shared between `apply_tiling_styles` and `render_inner_border`.
fn apply_corner_rounding<E: gpui::Styled + gpui::prelude::FluentBuilder>(
    element: E,
    tiling: Tiling,
) -> E {
    element
        .when(!(tiling.top || tiling.right), |d| {
            d.rounded_tr(BORDER_RADIUS)
        })
        .when(!(tiling.top || tiling.left), |d| {
            d.rounded_tl(BORDER_RADIUS)
        })
}

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
    /// style based on which resize edge the mouse is over. The wrapper div has
    /// a debug selector for testing.
    fn create_resize_canvas() -> impl IntoElement {
        div().id("resize-canvas").size_full().absolute().child(
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
            .size_full(),
        )
    }

    /// Apply tiling-aware padding and corner rounding to the outer div.
    fn apply_tiling_styles(div: Stateful<Div>, tiling: Tiling) -> Stateful<Div> {
        apply_corner_rounding(div, tiling)
            .when(!tiling.top, |d| d.pt(SHADOW_SIZE))
            .when(!tiling.bottom, |d| d.pb(SHADOW_SIZE))
            .when(!tiling.left, |d| d.pl(SHADOW_SIZE))
            .when(!tiling.right, |d| d.pr(SHADOW_SIZE))
    }

    /// Render the inner content border with styling based on tiling state.
    fn render_inner_border(self, tiling: Tiling, app: &App) -> Div {
        let inner_div = apply_corner_rounding(div(), tiling);

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

/// Build the backdrop element for client-side decorations.
///
/// This handles resize canvas insertion and mouse handler attachment,
/// both of which are conditional on the maximized state.
fn build_client_backdrop(tiling: Tiling, is_maximized: bool) -> Stateful<Div> {
    let mut outer = div()
        .id("gauss-window-backdrop")
        .bg(gpui::transparent_black());

    // Only add resize hit detection when NOT maximized.
    if !is_maximized {
        outer = outer.child(GaussWindowBorder::create_resize_canvas());
    }

    let styled_outer = GaussWindowBorder::apply_tiling_styles(outer, tiling);
    // Add resize handler only when not maximized
    styled_outer.when(!is_maximized, |d| {
        d.on_mouse_down(MouseButton::Left, on_resize_mouse_down)
    })
}

/// Build the inner content element for server-side decorations.
fn build_server_inner(children: Vec<AnyElement>, app: &App) -> Div {
    div()
        .on_mouse_move(|_e, _, ctx| ctx.stop_propagation())
        .bg(app.theme().background)
        .size_full()
        .children(children)
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
            Decorations::Client { tiling, .. } => build_client_backdrop(tiling, is_maximized),
        };

        let inner = match decorations {
            Decorations::Server => build_server_inner(self.children, app),
            Decorations::Client { tiling } => self.render_inner_border(tiling, app),
        };

        styled_backdrop.size_full().child(inner)
    }
}

/// Handle mouse-down for window resize initiation.
fn on_resize_mouse_down(_: &gpui::MouseDownEvent, window: &mut Window, _: &mut App) {
    // Defensive check: this handler is only attached when not maximized (see
    // RenderOnce impl), but re-check here in case of stale render state.
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

/// Flags indicating which edge zones the mouse position overlaps.
#[expect(
    clippy::struct_excessive_bools,
    reason = "four independent bools naturally represent the four window edge zones"
)]
struct EdgeZones {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl EdgeZones {
    /// Create edge zones from a mouse position and window dimensions.
    fn from_position(pos: Point<Pixels>, shadow_size: Pixels, size: Size<Pixels>) -> Self {
        Self {
            top: pos.y < shadow_size,
            bottom: pos.y > size.height - shadow_size,
            left: pos.x < shadow_size,
            right: pos.x > size.width - shadow_size,
        }
    }

    /// Check if the mouse is over a corner resize zone.
    ///
    /// Corners are formed by the intersection of two edge zones.
    const fn check_corner(&self) -> Option<ResizeEdge> {
        match (self.top, self.bottom, self.left, self.right) {
            (true, _, true, _) => Some(ResizeEdge::TopLeft),
            (true, _, _, true) => Some(ResizeEdge::TopRight),
            (_, true, true, _) => Some(ResizeEdge::BottomLeft),
            (_, true, _, true) => Some(ResizeEdge::BottomRight),
            _ => None,
        }
    }

    /// Check if the mouse is over an edge (non-corner) resize zone.
    const fn check_edge(&self) -> Option<ResizeEdge> {
        if self.top {
            Some(ResizeEdge::Top)
        } else if self.bottom {
            Some(ResizeEdge::Bottom)
        } else if self.left {
            Some(ResizeEdge::Left)
        } else if self.right {
            Some(ResizeEdge::Right)
        } else {
            None
        }
    }
}

/// Determine which resize edge (if any) the mouse position is over.
///
/// Corners take precedence at intersections (e.g., top-left wins over top).
/// Ported from `gpui-component::window_border` with identical logic.
fn resize_edge(pos: Point<Pixels>, shadow_size: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let zones = EdgeZones::from_position(pos, shadow_size, size);

    // Corners take precedence at intersections
    zones.check_corner().or_else(|| zones.check_edge())
}

/// Window root wrapper that renders content inside our custom window border.
///
/// GPUI's `App::open_window` requires the root view to implement `Render`,
/// which means we cannot use a bare helper function. This thin wrapper
/// exists solely to satisfy that constraint while delegating all layout
/// to `GaussWindowBorder`.
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
