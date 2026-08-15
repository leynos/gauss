//! Coordinate conversion shared by GPUI selection integration tests.

use gauss::model::{Vec2, Viewport};
use gpui::{Pixels, Point, point, px};

/// Convert a world-space point into GPUI screen coordinates.
#[must_use]
pub const fn viewport_to_screen_point(viewport: Viewport, world: Vec2) -> Point<Pixels> {
    let screen = viewport.world_to_screen(world);
    point(px(screen.x), px(screen.y))
}
