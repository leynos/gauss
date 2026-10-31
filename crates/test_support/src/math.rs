//! Small maths helpers for tests.

/// Return the midpoint between `a` and `b`.
#[must_use]
pub const fn midpoint(a: f32, b: f32) -> f32 {
    f32::midpoint(a, b)
}
