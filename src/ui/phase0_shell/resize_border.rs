//! Invisible resize border regions for client-side window resizing.
//!
//! This module provides resize handles at all window edges and corners,
//! allowing users to resize the window via drag operations. The regions
//! are invisible but intercept mouse events to initiate compositor-controlled
//! resize operations.
//!
//! On Linux, window resize hit regions are provided by `GaussWindowBorder`
//! (which handles the window shadow area), so this module returns an empty
//! vector to avoid overlapping hitboxes.

#[cfg(not(target_os = "linux"))]
use gpui::{AnyElement, CursorStyle, MouseButton, ResizeEdge, Window, div, prelude::*, px};

#[cfg(target_os = "linux")]
use gpui::AnyElement;

/// Width/height of the invisible resize edge region in pixels.
#[cfg(not(target_os = "linux"))]
const RESIZE_EDGE_SIZE: f32 = 6.0;

/// Width/height of the invisible resize corner region in pixels.
/// Corners are larger than edges to make them easier to target.
#[cfg(not(target_os = "linux"))]
const RESIZE_CORNER_SIZE: f32 = 12.0;

/// Render resize border regions around the window content.
///
/// Returns a vector of absolutely-positioned resize regions that should be
/// added directly as children of a relative-positioned container. By avoiding
/// an intermediate wrapper element, cursor state updates correctly when the
/// mouse enters from outside the window boundary.
///
/// On Linux, resize hit regions are provided by `GaussWindowBorder`, so this
/// returns an empty vector to avoid overlapping hitboxes.
///
/// # Example
///
/// ```ignore
/// div()
///     .relative()
///     .child(content)
///     .children(resize_borders())
/// ```
#[cfg(target_os = "linux")]
pub(super) const fn resize_borders() -> Vec<AnyElement> {
    Vec::new()
}

/// Render resize border regions around the window content.
///
/// Returns a vector of absolutely-positioned resize regions that should be
/// added directly as children of a relative-positioned container. By avoiding
/// an intermediate wrapper element, cursor state updates correctly when the
/// mouse enters from outside the window boundary.
///
/// # Example
///
/// ```ignore
/// div()
///     .relative()
///     .child(content)
///     .children(resize_borders())
/// ```
#[cfg(not(target_os = "linux"))]
pub(super) fn resize_borders() -> Vec<AnyElement> {
    vec![
        resize_edge(ResizeEdge::Top).into_any_element(),
        resize_edge(ResizeEdge::Bottom).into_any_element(),
        resize_edge(ResizeEdge::Left).into_any_element(),
        resize_edge(ResizeEdge::Right).into_any_element(),
        // Corners must come after edges so they have higher z-order and properly
        // capture events at the intersections.
        resize_corner(ResizeEdge::TopLeft).into_any_element(),
        resize_corner(ResizeEdge::TopRight).into_any_element(),
        resize_corner(ResizeEdge::BottomLeft).into_any_element(),
        resize_corner(ResizeEdge::BottomRight).into_any_element(),
    ]
}

#[cfg(not(target_os = "linux"))]
fn resize_edge(edge: ResizeEdge) -> impl gpui::IntoElement {
    let edge_size = px(RESIZE_EDGE_SIZE);
    let corner_size = px(RESIZE_CORNER_SIZE);
    let (id, cursor) = match edge {
        ResizeEdge::Top => ("resize-edge-top", CursorStyle::ResizeUpDown),
        ResizeEdge::Bottom => ("resize-edge-bottom", CursorStyle::ResizeUpDown),
        ResizeEdge::Left => ("resize-edge-left", CursorStyle::ResizeLeftRight),
        ResizeEdge::Right => ("resize-edge-right", CursorStyle::ResizeLeftRight),
        _ => ("resize-edge-unknown", CursorStyle::Arrow),
    };

    let mut el = div()
        .id(id)
        .debug_selector(move || format!("#{id}"))
        .absolute()
        .cursor(cursor);

    // Edges are inset by the corner size to avoid overlapping with corners.
    el = match edge {
        ResizeEdge::Top => el.top_0().left(corner_size).right(corner_size).h(edge_size),
        ResizeEdge::Bottom => el
            .bottom_0()
            .left(corner_size)
            .right(corner_size)
            .h(edge_size),
        ResizeEdge::Left => el
            .left_0()
            .top(corner_size)
            .bottom(corner_size)
            .w(edge_size),
        ResizeEdge::Right => el
            .right_0()
            .top(corner_size)
            .bottom(corner_size)
            .w(edge_size),
        _ => el,
    };

    el.on_mouse_down(
        MouseButton::Left,
        move |_event, window: &mut Window, _cx| {
            // Don't start resize if window is maximized (stale render state)
            if !window.is_maximized() {
                window.start_window_resize(edge);
            }
        },
    )
}

#[cfg(not(target_os = "linux"))]
fn resize_corner(edge: ResizeEdge) -> impl gpui::IntoElement {
    let corner_size = px(RESIZE_CORNER_SIZE);
    let (id, cursor) = match edge {
        ResizeEdge::TopLeft => ("resize-corner-tl", CursorStyle::ResizeUpLeftDownRight),
        ResizeEdge::TopRight => ("resize-corner-tr", CursorStyle::ResizeUpRightDownLeft),
        ResizeEdge::BottomLeft => ("resize-corner-bl", CursorStyle::ResizeUpRightDownLeft),
        ResizeEdge::BottomRight => ("resize-corner-br", CursorStyle::ResizeUpLeftDownRight),
        _ => ("resize-corner-unknown", CursorStyle::Arrow),
    };

    let base = div()
        .id(id)
        .debug_selector(move || format!("#{id}"))
        .absolute()
        .w(corner_size)
        .h(corner_size)
        .cursor(cursor);

    let el = match edge {
        ResizeEdge::TopLeft => base.top_0().left_0(),
        ResizeEdge::TopRight => base.top_0().right_0(),
        ResizeEdge::BottomLeft => base.bottom_0().left_0(),
        ResizeEdge::BottomRight => base.bottom_0().right_0(),
        _ => base,
    };

    el.on_mouse_down(
        MouseButton::Left,
        move |_event, window: &mut Window, _cx| {
            // Don't start resize if window is maximized (stale render state)
            if !window.is_maximized() {
                window.start_window_resize(edge);
            }
        },
    )
}
