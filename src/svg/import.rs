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
    Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2, parse_hex_rgb,
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

    if !trimmed.starts_with('#') {
        return Err(SvgImportError::InvalidColour);
    }

    parse_hex_rgb(trimmed)
        .map(Some)
        .map_err(|_| SvgImportError::InvalidColour)
}

fn parse_opacity_to_alpha(value: &str) -> Result<u8, SvgImportError> {
    let s = value.trim();
    if s.is_empty() {
        return Err(SvgImportError::InvalidOpacity);
    }

    let opacity = s
        .parse::<f32>()
        .map_err(|_| SvgImportError::InvalidOpacity)?;
    if !(0.0..=1.0).contains(&opacity) {
        return Err(SvgImportError::InvalidOpacity);
    }

    let scaled = (opacity * 255.0).round();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "opacity is validated to 0..=1 so the scaled value fits in u8"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "opacity is validated as non-negative before scaling"
    )]
    let alpha = scaled as u8;
    Ok(alpha)
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
    for quote in ['"', '\''] {
        let needle = format!("{name}={quote}");
        let Some(start) = tag.find(&needle) else {
            continue;
        };
        let Some(after) = tag.get((start + needle.len())..) else {
            continue;
        };
        let Some(end) = after.find(quote) else {
            continue;
        };
        if let Some(value) = after.get(..end) {
            return Some(value.to_owned());
        }
    }

    None
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
    let mut last_segment = None;

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
                last_segment = Some(SegmentKind::Line);
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
                last_segment = Some(SegmentKind::Cubic);
                geom.anchors.push(Anchor {
                    pos: Vec2::new(x, y),
                    handle_in: Some(Vec2::new(x2, y2)),
                    handle_out: None,
                });
            }
            'Z' => {
                geom.closed = true;
                geom.closing_segment = last_segment.unwrap_or(SegmentKind::Line);
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

        if ch == '-' || ch == '+' {
            if number_buf.is_empty() {
                number_buf.push(ch);
                continue;
            }

            let is_exponent = number_buf.ends_with('e') || number_buf.ends_with('E');
            if is_exponent {
                number_buf.push(ch);
            } else {
                flush_number(&mut number_buf, &mut tokens)?;
                number_buf.push(ch);
            }
            continue;
        }

        number_buf.push(ch);
    }
    flush_number(&mut number_buf, &mut tokens)?;

    Ok(tokens)
}
