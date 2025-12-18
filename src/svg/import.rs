//! SVG import for the Gauss document model.
//!
//! This importer is deliberately limited. It is intended to round-trip SVG
//! emitted by `crate::svg::export::export_svg`, and to accept many simple SVGs.
//!
//! Supported subset:
//!
//! - `<path>` elements only (other elements are ignored)
//! - absolute commands: `M`, `L`, `C`, `Z`
//! - attributes: `d`, `stroke`, `stroke-width`, `stroke-opacity`, `fill`,
//!   `fill-opacity`

use std::{error::Error, fmt};

use crate::model::{
    Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2,
};

/// Errors returned by [`import_svg`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SvgImportError {
    /// The input did not contain valid UTF-8 XML-like data we can parse.
    MalformedSvg,
    /// A `<path>` element was missing its required `d` attribute.
    MissingPathData,
    /// A path `d` attribute contained an unsupported command.
    UnsupportedPathCommand(char),
    /// A path `d` attribute was syntactically invalid.
    InvalidPathData,
    /// A colour attribute was present but not understood.
    InvalidColour,
    /// A numeric attribute was present but not understood.
    InvalidNumber,
    /// An opacity attribute was present but not understood.
    InvalidOpacity,
}

impl fmt::Display for SvgImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedSvg => write!(f, "malformed SVG"),
            Self::MissingPathData => write!(f, "missing path data attribute"),
            Self::UnsupportedPathCommand(cmd) => write!(f, "unsupported path command: {cmd}"),
            Self::InvalidPathData => write!(f, "invalid path data"),
            Self::InvalidColour => write!(f, "invalid colour"),
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidOpacity => write!(f, "invalid opacity"),
        }
    }
}

impl Error for SvgImportError {}

/// Import an SVG string as a [`Document`].
///
/// Only `<path>` elements are imported. Paths with missing/invalid data are
/// rejected with an error.
///
/// # Errors
///
/// Returns an [`SvgImportError`] when the input is not valid SVG, contains
/// unsupported path commands, or uses attributes outside the supported subset.
pub fn import_svg(svg: &str) -> Result<Document, SvgImportError> {
    let mut shapes = Vec::new();

    for (index, tag) in extract_path_tags(svg).into_iter().enumerate() {
        let d = attribute_value(&tag, "d").ok_or(SvgImportError::MissingPathData)?;

        let (stroke, stroke_alpha) = parse_paint_with_opacity(&tag, "stroke", "stroke-opacity")?;
        let (fill, fill_alpha) = parse_paint_with_opacity(&tag, "fill", "fill-opacity")?;

        let stroke_width = attribute_value(&tag, "stroke-width")
            .map(|value| {
                value
                    .parse::<f32>()
                    .map_err(|_| SvgImportError::InvalidNumber)
            })
            .transpose()?
            .unwrap_or(1.0);

        let path = parse_path_data(&d)?;

        let style = PaintStyle::new(
            stroke.map(|rgb| with_alpha(rgb, stroke_alpha)),
            stroke_width,
            fill.map(|rgb| with_alpha(rgb, fill_alpha)),
        );

        shapes.push(Shape {
            id: ShapeId::new_v4(),
            z: index.try_into().map_err(|_| SvgImportError::MalformedSvg)?,
            style,
            path,
        });
    }

    Ok(Document { shapes })
}

const fn with_alpha(mut rgb: Rgba, alpha: u8) -> Rgba {
    rgb.a = alpha;
    rgb
}

fn parse_paint_with_opacity(
    tag: &str,
    paint_attr: &str,
    opacity_attr: &str,
) -> Result<(Option<Rgba>, u8), SvgImportError> {
    let paint = match attribute_value(tag, paint_attr) {
        None => None,
        Some(value) => parse_colour(&value)?,
    };

    let alpha = match attribute_value(tag, opacity_attr) {
        None => 255,
        Some(value) => parse_opacity_to_alpha(&value)?,
    };

    Ok((paint, alpha))
}

fn parse_colour(value: &str) -> Result<Option<Rgba>, SvgImportError> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("none") {
        return Ok(None);
    }

    let Some(hex) = trimmed.strip_prefix('#') else {
        return Err(SvgImportError::InvalidColour);
    };

    if hex.len() != 6 {
        return Err(SvgImportError::InvalidColour);
    }

    let mut chars = hex.chars();
    let r = parse_hex_byte(&mut chars)?;
    let g = parse_hex_byte(&mut chars)?;
    let b = parse_hex_byte(&mut chars)?;
    if chars.next().is_some() {
        return Err(SvgImportError::InvalidColour);
    }

    Ok(Some(Rgba::new(r, g, b, 255)))
}

fn parse_hex_byte(chars: &mut impl Iterator<Item = char>) -> Result<u8, SvgImportError> {
    let high = chars
        .next()
        .and_then(|ch| ch.to_digit(16))
        .ok_or(SvgImportError::InvalidColour)?;
    let low = chars
        .next()
        .and_then(|ch| ch.to_digit(16))
        .ok_or(SvgImportError::InvalidColour)?;
    let value = (high << 4) | low;
    u8::try_from(value).map_err(|_| SvgImportError::InvalidColour)
}

fn parse_opacity_to_alpha(value: &str) -> Result<u8, SvgImportError> {
    let s = value.trim();
    if s.is_empty() {
        return Err(SvgImportError::InvalidOpacity);
    }

    let (int_part_str, frac_part_str) = match s.split_once('.') {
        Some((int_part, frac_part)) => (int_part, Some(frac_part)),
        None => (s, None),
    };

    let int_part_value = match int_part_str {
        "" | "0" => 0u8,
        "1" => 1u8,
        _ => return Err(SvgImportError::InvalidOpacity),
    };

    let frac_part = frac_part_str.unwrap_or("");
    if int_part_value == 1 && !frac_part.is_empty() {
        return Err(SvgImportError::InvalidOpacity);
    }

    if int_part_value == 1 {
        return Ok(255);
    }

    if frac_part.is_empty() {
        return Ok(0);
    }

    let mut denom: u32 = 1;
    let mut numer: u32 = 0;
    for ch in frac_part.chars() {
        let digit = ch.to_digit(10).ok_or(SvgImportError::InvalidOpacity)?;
        denom = denom
            .checked_mul(10)
            .ok_or(SvgImportError::InvalidOpacity)?;
        numer = numer
            .checked_mul(10)
            .and_then(|v| v.checked_add(digit))
            .ok_or(SvgImportError::InvalidOpacity)?;
    }

    let scaled = numer
        .checked_mul(255)
        .ok_or(SvgImportError::InvalidOpacity)?;
    let denom_half = denom.checked_div(2).ok_or(SvgImportError::InvalidOpacity)?;
    let rounded = scaled
        .checked_add(denom_half)
        .ok_or(SvgImportError::InvalidOpacity)?
        .checked_div(denom)
        .ok_or(SvgImportError::InvalidOpacity)?;

    u8::try_from(rounded).map_err(|_| SvgImportError::InvalidOpacity)
}

fn extract_path_tags(svg: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut remaining = svg;

    while let Some(start) = remaining.find("<path") {
        let Some(after_start) = remaining.get(start..) else {
            break;
        };
        let Some(end) = after_start.find('>') else {
            break;
        };
        let Some(tag) = after_start.get(..=end) else {
            break;
        };
        tags.push(tag.to_owned());
        remaining = after_start.get((end + 1)..).unwrap_or_default();
    }

    tags
}

fn attribute_value(tag: &str, name: &str) -> Option<String> {
    let needle = format!(r#"{name}=""#);
    let start = tag.find(&needle)?;
    let after = tag.get((start + needle.len())..)?;
    let end = after.find('"')?;
    after.get(..end).map(ToOwned::to_owned)
}

#[derive(Clone, Debug, PartialEq)]
enum PathToken {
    Command(char),
    Number(f32),
}

fn parse_path_data(d: &str) -> Result<PathGeom, SvgImportError> {
    let tokens = tokenize_path_data(d)?;
    let mut it = tokens.into_iter().peekable();

    let mut geom = PathGeom::new();
    geom.closing_segment = SegmentKind::Line;

    while let Some(token) = it.next() {
        let PathToken::Command(cmd) = token else {
            return Err(SvgImportError::InvalidPathData);
        };

        match cmd {
            'M' => {
                let x = next_number(&mut it)?;
                let y = next_number(&mut it)?;
                geom.anchors.push(Anchor::new(Vec2::new(x, y)));
            }
            'L' => {
                let x = next_number(&mut it)?;
                let y = next_number(&mut it)?;
                geom.segments.push(SegmentKind::Line);
                geom.anchors.push(Anchor::new(Vec2::new(x, y)));
            }
            'C' => {
                let x1 = next_number(&mut it)?;
                let y1 = next_number(&mut it)?;
                let x2 = next_number(&mut it)?;
                let y2 = next_number(&mut it)?;
                let x = next_number(&mut it)?;
                let y = next_number(&mut it)?;

                let Some(prev) = geom.anchors.last_mut() else {
                    return Err(SvgImportError::InvalidPathData);
                };
                prev.handle_out = Some(Vec2::new(x1, y1));
                geom.segments.push(SegmentKind::Cubic);
                geom.anchors.push(Anchor {
                    pos: Vec2::new(x, y),
                    handle_in: Some(Vec2::new(x2, y2)),
                    handle_out: None,
                });
            }
            'Z' => {
                geom.closed = true;
                geom.closing_segment = SegmentKind::Line;
            }
            other => return Err(SvgImportError::UnsupportedPathCommand(other)),
        }
    }

    Ok(geom)
}

fn next_number<I>(it: &mut I) -> Result<f32, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    match it.next() {
        Some(PathToken::Number(v)) => Ok(v),
        _ => Err(SvgImportError::InvalidPathData),
    }
}

fn tokenize_path_data(d: &str) -> Result<Vec<PathToken>, SvgImportError> {
    fn flush_number(
        number_buf: &mut String,
        tokens: &mut Vec<PathToken>,
    ) -> Result<(), SvgImportError> {
        if number_buf.is_empty() {
            return Ok(());
        }
        let value = number_buf
            .parse::<f32>()
            .map_err(|_| SvgImportError::InvalidPathData)?;
        tokens.push(PathToken::Number(value));
        number_buf.clear();
        Ok(())
    }

    let mut tokens = Vec::new();
    let mut number_buf = String::new();

    for ch in d.chars() {
        if ch.is_ascii_alphabetic() {
            flush_number(&mut number_buf, &mut tokens)?;
            tokens.push(PathToken::Command(ch));
            continue;
        }

        if ch.is_ascii_whitespace() || ch == ',' {
            flush_number(&mut number_buf, &mut tokens)?;
            continue;
        }

        number_buf.push(ch);
    }
    flush_number(&mut number_buf, &mut tokens)?;

    Ok(tokens)
}
