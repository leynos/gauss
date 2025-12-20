//! Viewport transforms (pan/zoom) for mapping world to screen coordinates.

#![expect(
    clippy::float_arithmetic,
    reason = "viewport transforms require floating-point maths"
)]

use crate::model::Vec2;

/// Viewport transform for mapping between world space and screen space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Pan offset in screen pixels, applied after scaling.
    pub pan: Vec2,
    /// Zoom scale factor (`1.0` is 100%).
    zoom: f32,
}

impl Viewport {
    /// Construct a default viewport with no pan and 1x zoom.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Return the current zoom scale factor.
    #[must_use]
    pub const fn zoom(self) -> f32 {
        self.zoom
    }

    /// Set the zoom scale factor, clamping it to a safe range.
    pub const fn set_zoom(&mut self, zoom: f32) {
        self.zoom = clamp_zoom(zoom);
    }

    /// Convert world coordinates to screen coordinates.
    #[must_use]
    pub const fn world_to_screen(self, p: Vec2) -> Vec2 {
        p.mul(self.zoom).add(self.pan)
    }

    /// Convert screen coordinates to world coordinates.
    #[must_use]
    pub const fn screen_to_world(self, p: Vec2) -> Vec2 {
        p.sub(self.pan).mul(1.0 / self.zoom)
    }

    /// Zoom around a cursor position in screen coordinates.
    ///
    /// This keeps the world point under the cursor stable as zoom changes.
    pub fn zoom_around_screen_point(&mut self, cursor_screen: Vec2, zoom_factor: f32) {
        let before = self.screen_to_world(cursor_screen);
        self.set_zoom(self.zoom * zoom_factor);
        self.pan = cursor_screen.sub(before.mul(self.zoom));
    }
}

const fn clamp_zoom(zoom: f32) -> f32 {
    const MIN_ZOOM: f32 = 0.05;
    const MAX_ZOOM: f32 = 64.0;

    if zoom < MIN_ZOOM {
        MIN_ZOOM
    } else if zoom > MAX_ZOOM {
        MAX_ZOOM
    } else {
        zoom
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn zoom_around_cursor_keeps_world_point_stable() {
        let mut viewport = Viewport::new();
        viewport.pan = Vec2::new(50.0, -25.0);
        viewport.set_zoom(2.0);

        let cursor = Vec2::new(400.0, 300.0);
        let world_before = viewport.screen_to_world(cursor);

        viewport.zoom_around_screen_point(cursor, 1.25);

        let world_after = viewport.screen_to_world(cursor);
        let error = world_before.distance(world_after);
        assert!(error < 0.001, "world point drifted by {error}");
    }
}
