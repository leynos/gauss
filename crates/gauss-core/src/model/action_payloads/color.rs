//! Colour types for action payloads.

/// RGB colour value for action payloads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

/// Construction helper for [`Color`].
///
/// `Rgb8` provides named fields for constructing a `Color` via `.into()`
/// or `Color::from`. It exists solely to make colour construction more
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

impl From<Rgb8> for Color {
    fn from(rgb: Rgb8) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

impl Color {
    /// Construct a new colour from RGB components.
    #[must_use]
    pub const fn new(rgb: Rgb8) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}
