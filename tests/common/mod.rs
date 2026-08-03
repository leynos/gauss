//! Shared helpers for Phase 0 GPUI integration tests.
//!
//! The helpers themselves live in [`shared_helpers`] and are re-exported here so
//! existing `use common::...` paths keep working. That module carries the
//! `dead_code` expectation rather than this one, because the expectation is only
//! defensible for the legacy shared surface: see its module documentation for
//! why the suppression cannot be narrowed to individual items.
mod shared_helpers;

pub use shared_helpers::*;

/// Convert a world-space point into GPUI screen coordinates.
pub const fn viewport_to_screen_point(
    viewport: gauss::model::Viewport,
    world: Vec2,
) -> Point<Pixels> {
    let screen = viewport.world_to_screen(world);
    point(px(screen.x), px(screen.y))
}
