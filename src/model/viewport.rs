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

    /// Adjust pan to maintain an anchor point after canvas resize.
    ///
    /// When the window is resized, this method adjusts the pan offset so that
    /// the world point at the anchor position remains at the same relative
    /// screen position.
    ///
    /// The `anchor_factor` determines which point on screen stays fixed:
    /// - (0.0, 0.0) = top-left corner
    /// - (1.0, 0.0) = top-right corner
    /// - (0.5, 0.5) = centre
    ///
    /// # Example
    ///
    /// ```
    /// use gauss::model::{Viewport, Vec2};
    ///
    /// let mut viewport = Viewport::new();
    /// viewport.pan = Vec2::new(50.0, 25.0);
    ///
    /// let old_size = Vec2::new(800.0, 600.0);
    /// let new_size = Vec2::new(1000.0, 700.0);
    ///
    /// // Keep top-left anchored
    /// viewport.adjust_pan_for_resize(old_size, new_size, (0.0, 0.0));
    /// ```
    pub fn adjust_pan_for_resize(
        &mut self,
        old_size: Vec2,
        new_size: Vec2,
        anchor_factor: (f32, f32),
    ) {
        let old_anchor_screen =
            Vec2::new(old_size.x * anchor_factor.0, old_size.y * anchor_factor.1);
        let anchor_world = self.screen_to_world(old_anchor_screen);
        let new_anchor_screen =
            Vec2::new(new_size.x * anchor_factor.0, new_size.y * anchor_factor.1);
        self.pan = new_anchor_screen.sub(anchor_world.mul(self.zoom));
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

    #[rstest]
    fn adjust_pan_preserves_top_left_anchor() {
        let mut viewport = Viewport::new();
        viewport.pan = Vec2::new(50.0, 25.0);
        viewport.set_zoom(2.0);

        let old_size = Vec2::new(800.0, 600.0);
        let new_size = Vec2::new(1000.0, 700.0);

        // World point at top-left before resize
        let world_at_origin_before = viewport.screen_to_world(Vec2::ZERO);

        viewport.adjust_pan_for_resize(old_size, new_size, (0.0, 0.0));

        // World point at top-left should be identical after resize
        let world_at_origin_after = viewport.screen_to_world(Vec2::ZERO);
        let error = world_at_origin_before.distance(world_at_origin_after);
        assert!(error < 0.001, "anchor drifted by {error}");
    }

    #[rstest]
    fn adjust_pan_preserves_centre_anchor() {
        let mut viewport = Viewport::new();
        viewport.pan = Vec2::new(100.0, 50.0);
        viewport.set_zoom(1.5);

        let old_size = Vec2::new(800.0, 600.0);
        let new_size = Vec2::new(1200.0, 800.0);

        // World point at centre before resize
        let old_centre = Vec2::new(400.0, 300.0);
        let world_at_centre_before = viewport.screen_to_world(old_centre);

        viewport.adjust_pan_for_resize(old_size, new_size, (0.5, 0.5));

        // World point at new centre should be same world coordinate
        let new_centre = Vec2::new(600.0, 400.0);
        let world_at_centre_after = viewport.screen_to_world(new_centre);
        let error = world_at_centre_before.distance(world_at_centre_after);
        assert!(error < 0.001, "centre anchor drifted by {error}");
    }

    #[rstest]
    fn adjust_pan_preserves_top_right_anchor() {
        let mut viewport = Viewport::new();
        viewport.pan = Vec2::new(50.0, 25.0);
        viewport.set_zoom(1.0);

        let old_size = Vec2::new(800.0, 600.0);
        let new_size = Vec2::new(1000.0, 700.0);

        // World point at top-right before resize
        let old_top_right = Vec2::new(800.0, 0.0);
        let world_at_top_right_before = viewport.screen_to_world(old_top_right);

        viewport.adjust_pan_for_resize(old_size, new_size, (1.0, 0.0));

        // World point at new top-right should be same world coordinate
        let new_top_right = Vec2::new(1000.0, 0.0);
        let world_at_top_right_after = viewport.screen_to_world(new_top_right);
        let error = world_at_top_right_before.distance(world_at_top_right_after);
        assert!(error < 0.001, "top-right anchor drifted by {error}");
    }
}
