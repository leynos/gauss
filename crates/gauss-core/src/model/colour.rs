//! Colour parsing and formatting helpers shared across the UI and SVG layers.
//!
//! This module keeps hex colour handling consistent so we do not diverge
//! between SVG import/export and the UI colour picker integration.

use crate::model::Rgba;

/// Errors returned when parsing hexadecimal RGB colour literals.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HexColourParseError {
    /// The input did not contain the expected number of hexadecimal digits.
    #[error("hex colour has an invalid length")]
    InvalidLength,
    /// The input contained a non-hexadecimal digit.
    #[error("hex colour contains an invalid digit")]
    InvalidDigit,
}

/// Format an RGB colour as a lowercase `#rrggbb` string.
///
/// The alpha channel (`colour.a`) is ignored; the output always contains
/// exactly three hex pairs for red, green, and blue.
#[must_use]
pub fn format_hex_rgb(colour: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
}

/// Parse a `#rrggbb` or `rrggbb` string into an opaque [`Rgba`] value.
///
/// # Errors
///
/// Returns [`HexColourParseError`] when the input is not valid hexadecimal RGB
/// text.
pub fn parse_hex_rgb(hex: &str) -> Result<Rgba, HexColourParseError> {
    parse_hex_colour(hex, false)
}

fn parse_hex_colour(hex: &str, allow_alpha: bool) -> Result<Rgba, HexColourParseError> {
    let without_hash = hex.strip_prefix('#').unwrap_or(hex);
    let len = without_hash.len();
    let expects_alpha = match len {
        6 => false,
        8 if allow_alpha => true,
        _ => return Err(HexColourParseError::InvalidLength),
    };

    let mut iter = without_hash.as_bytes().iter().copied();
    let r = parse_hex_byte(&mut iter)?;
    let g = parse_hex_byte(&mut iter)?;
    let b = parse_hex_byte(&mut iter)?;
    let a = if expects_alpha {
        parse_hex_byte(&mut iter)?
    } else {
        255
    };

    Ok(Rgba::new(r, g, b, a))
}

fn parse_hex_byte(iter: &mut impl Iterator<Item = u8>) -> Result<u8, HexColourParseError> {
    let high_digit = parse_hex_nibble(iter.next().ok_or(HexColourParseError::InvalidLength)?)?;
    let low_digit = parse_hex_nibble(iter.next().ok_or(HexColourParseError::InvalidLength)?)?;
    Ok((high_digit << 4) | low_digit)
}

const fn parse_hex_nibble(byte: u8) -> Result<u8, HexColourParseError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(HexColourParseError::InvalidDigit),
    }
}
