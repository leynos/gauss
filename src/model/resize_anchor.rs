//! Resize anchor configuration for viewport adjustment during window resize.
//!
//! When the window is resized, the viewport pan must be adjusted to keep
//! content anchored at a specific point. This module provides configuration
//! for which anchor point to use.

/// Where the canvas content should remain anchored during resize.
///
/// When the window is resized, the viewport adjusts so that the world point
/// at the anchor position remains at the same relative screen position.
///
/// # Example
///
/// With `TopLeft` anchor, dragging the bottom-right corner to enlarge the
/// window will keep the top-left content fixed while the canvas extends
/// down and to the right.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResizeAnchor {
    /// Top-left corner (default for LTR locales).
    #[default]
    TopLeft,
    /// Top-right corner (default for RTL locales).
    TopRight,
    /// Centre of the canvas.
    Centre,
}

impl ResizeAnchor {
    /// Returns the anchor point as a normalised factor (0..1) for each axis.
    ///
    /// (0, 0) = top-left, (1, 1) = bottom-right.
    #[must_use]
    pub const fn as_factor(self) -> (f32, f32) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::Centre => (0.5, 0.5),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for resize anchor normalisation.

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(ResizeAnchor::TopLeft, (0.0, 0.0))]
    #[case(ResizeAnchor::TopRight, (1.0, 0.0))]
    #[case(ResizeAnchor::Centre, (0.5, 0.5))]
    fn as_factor_returns_expected_values(
        #[case] anchor: ResizeAnchor,
        #[case] expected: (f32, f32),
    ) {
        assert_eq!(anchor.as_factor(), expected);
    }
}
