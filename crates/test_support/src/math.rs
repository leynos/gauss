//! Small maths helpers for tests.

/// Return the midpoint between `a` and `b`.
#[must_use]
#[expect(
    clippy::float_arithmetic,
    reason = "tests need midpoint calculations for positioning gestures"
)]
pub const fn midpoint(a: f32, b: f32) -> f32 {
    (a + b) * 0.5
}
