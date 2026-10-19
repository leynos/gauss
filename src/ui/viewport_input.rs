//! Viewport input mapping for Phase 0.
//!
//! Phase 0 keeps viewport input handling narrowly scoped to:
//!
//! - scroll wheel pan (with horizontal support), and
//! - Ctrl/Cmd + wheel zoom around cursor.
//!
//! This module is intentionally small and testable. It exposes a single helper
//! that mutates a `crate::model::Viewport` based on a `gpui::ScrollWheelEvent`.

#![expect(
    clippy::float_arithmetic,
    reason = "viewport input maps scroll deltas into floating-point zoom/pan"
)]

use gpui::{Pixels, ScrollWheelEvent};

use crate::model::{Vec2, Viewport};

/// Exponential zoom-rate multiplier applied to each vertical scroll pixel.
///
/// A positive 100-pixel vertical delta contributes `exp(0.1)` before the zoom clamp.
const ZOOM_FACTOR_PER_PIXEL: f32 = 0.001;

/// Detects shift-scroll intended for horizontal panning.
///
/// Returns true when shift is held and the scroll delta is purely vertical,
/// indicating the user intends to pan horizontally (common trackpad gesture).
fn is_shift_vertical_scroll(event: &ScrollWheelEvent, dx: f32, dy: f32) -> bool {
    event.shift && dx.abs() <= f32::EPSILON && dy.abs() > f32::EPSILON
}

/// Applies a scroll event as cursor-centred zoom or viewport panning.
///
/// Secondary-modified scroll zooms at the cursor when the factor changes; shift
/// plus a purely vertical delta pans horizontally. Other non-negligible deltas
/// pan directly; a zero or negligible delta returns `false`.
pub(super) fn apply_scroll_wheel_event(
    viewport: &mut Viewport,
    event: &ScrollWheelEvent,
    line_height: Pixels,
) -> bool {
    let delta = event.delta.pixel_delta(line_height);
    let mut dx = f32::from(delta.x);
    let mut dy = f32::from(delta.y);

    if event.secondary() {
        let zoom_factor = zoom_factor_from_scroll(dy);
        if (zoom_factor - 1.0).abs() > f32::EPSILON {
            let cursor = Vec2::new(f32::from(event.position.x), f32::from(event.position.y));
            viewport.zoom_around_screen_point(cursor, zoom_factor);
            return true;
        }

        return false;
    }

    if is_shift_vertical_scroll(event, dx, dy) {
        dx = dy;
        dy = 0.0;
    }

    if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
        return false;
    }

    viewport.pan = viewport.pan.add(Vec2::new(dx, dy));
    true
}

/// Converts a vertical pixel delta to an exponential zoom factor clamped to
/// `0.5..=2.0`.
///
/// A delta of 100 pixels produces `exp(0.1)` (about `1.105`); larger deltas
/// cannot zoom beyond the clamp.
fn zoom_factor_from_scroll(delta_y_pixels: f32) -> f32 {
    let raw = (delta_y_pixels * ZOOM_FACTOR_PER_PIXEL).exp();
    raw.clamp(0.5, 2.0)
}
