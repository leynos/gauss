//! Colour parsing and formatting helpers shared across the UI and SVG layers.
//!
//! This module keeps hex colour handling consistent so we do not diverge
//! between SVG import/export and the UI colour picker integration.

use crate::model::Rgba;

#[derive(Clone, Copy, Debug)]
pub(crate) enum HexColourParseError {
    InvalidLength,
    InvalidDigit,
}

pub(crate) fn format_hex_rgb(colour: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
}

pub(crate) fn format_hex_rgba_optional_alpha(colour: Rgba) -> String {
    if colour.a == 255 {
        return format_hex_rgb(colour);
    }

    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        colour.r, colour.g, colour.b, colour.a
    )
}

pub(crate) fn parse_hex_rgb(hex: &str) -> Result<Rgba, HexColourParseError> {
    parse_hex_colour(hex, false)
}

pub(crate) fn parse_hex_rgba(hex: &str) -> Result<Rgba, HexColourParseError> {
    parse_hex_colour(hex, true)
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

    if iter.next().is_some() {
        return Err(HexColourParseError::InvalidLength);
    }

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
