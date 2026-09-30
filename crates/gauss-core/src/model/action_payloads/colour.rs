//! Colour types for action payloads.

/// RGB colour value for action payloads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Colour {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

/// Construction helper for [`Colour`].
///
/// `Rgb8` provides named fields for constructing a `Colour` via `.into()`
/// or `Colour::from`. It exists solely to make colour construction more
/// explicit at call sites.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgb8 {
    /// Red channel (0-255).
    pub r: u8,
    /// Green channel (0-255).
    pub g: u8,
    /// Blue channel (0-255).
    pub b: u8,
}

impl From<Rgb8> for Colour {
    fn from(rgb: Rgb8) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

impl Colour {
    /// Construct a new colour from RGB components.
    ///
    /// # Examples
    ///
    /// ```
    /// use gauss_core::model::action::{Colour, Rgb8};
    ///
    /// let colour = Colour::new(Rgb8 { r: 12, g: 34, b: 56 });
    /// assert_eq!((colour.r, colour.g, colour.b), (12, 34, 56));
    /// ```
    #[must_use]
    pub const fn new(rgb: Rgb8) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}
