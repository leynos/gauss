//! Small maths helpers for tests.

/// Return the midpoint between `a` and `b`.
#[must_use]
pub const fn midpoint(a: f32, b: f32) -> f32 {
    f32::midpoint(a, b)
}

#[cfg(test)]
mod tests {
    //! Compile-time and runtime checks of the maths helpers.

    use super::midpoint;

    /// The midpoint evaluated at compile time, so the helper stays usable in a `const`.
    const HALF: f32 = midpoint(0.0, 1.0);

    // Bit equality, because `float_cmp` is denied and a const context needs no tolerance here.
    const _: () = assert!(HALF.to_bits() == 0.5_f32.to_bits());

    #[test]
    fn the_midpoint_is_the_same_at_run_time() {
        assert_eq!(midpoint(0.0, 1.0).to_bits(), HALF.to_bits());
    }
}
