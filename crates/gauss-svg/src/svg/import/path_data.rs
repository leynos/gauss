//! SVG path-data parsing helpers.

use super::SvgImportError;
use crate::model::{Anchor, PathGeom, SegmentKind, Vec2};

/// Lexical token produced while scanning an SVG path-data string.
#[derive(Clone, Debug, PartialEq)]
enum PathToken {
    /// A path command letter such as `M`, `L`, `C`, or `Z`.
    Command(char),
    /// A numeric coordinate or control-point component.
    Number(f32),
}

/// Parse an SVG `d` attribute into path geometry.
///
/// Only the absolute command subset emitted by Gauss export is accepted;
/// relative commands and unknown letters yield an error.
pub(super) fn parse_path_data(d: &str) -> Result<PathGeom, SvgImportError> {
    let tokens = tokenize_path_data(d)?;
    let mut it = tokens.into_iter().peekable();

    let mut geom = PathGeom::new();
    let mut last_segment = None;

    while let Some(token) = it.next() {
        let PathToken::Command(cmd) = token else {
            return Err(SvgImportError::InvalidPathData);
        };

        match cmd {
            'M' => parse_move_command(&mut it, &mut geom)?,
            'L' => last_segment = Some(parse_line_command(&mut it, &mut geom)?),
            'C' => last_segment = Some(parse_cubic_command(&mut it, &mut geom)?),
            'Z' => close_path(&mut geom, last_segment),
            other => return Err(SvgImportError::UnsupportedPathCommand(other)),
        }
    }

    Ok(geom)
}

/// Consume the next two number tokens and pair them as a 2D point.
fn next_vec2<I>(it: &mut I) -> Result<Vec2, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    let x = next_number(it)?;
    let y = next_number(it)?;
    Ok(Vec2::new(x, y))
}

/// Handle `M`: append a fresh anchor with no outgoing handles.
fn parse_move_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<(), SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    geom.anchors.push(Anchor::new(next_vec2(it)?));
    Ok(())
}

/// Handle `L`: record a line segment and append its end anchor.
fn parse_line_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<SegmentKind, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    geom.segments.push(SegmentKind::Line);
    geom.anchors.push(Anchor::new(next_vec2(it)?));
    Ok(SegmentKind::Line)
}

/// Handle `C`: attach the outgoing handle to the previous anchor and append a
/// cubic segment ending at a new anchor carrying the incoming handle.
///
/// Fails when no previous anchor exists to receive the outgoing handle.
fn parse_cubic_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<SegmentKind, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    let handle_out = next_vec2(it)?;
    let handle_in = next_vec2(it)?;
    let pos = next_vec2(it)?;

    let prev = geom
        .anchors
        .last_mut()
        .ok_or(SvgImportError::InvalidPathData)?;
    prev.handle_out = Some(handle_out);
    geom.segments.push(SegmentKind::Cubic);
    geom.anchors.push(Anchor {
        pos,
        handle_in: Some(handle_in),
        handle_out: None,
    });
    Ok(SegmentKind::Cubic)
}

/// Handle `Z`: mark the path closed, remembering the segment type to close with.
///
/// A path closed before any segment defaults to a line-closing segment.
fn close_path(geom: &mut PathGeom, last_segment: Option<SegmentKind>) {
    geom.closed = true;
    geom.closing_segment = last_segment.unwrap_or(SegmentKind::Line);
}

/// Consume the next token, requiring it to be a number.
fn next_number<I>(it: &mut I) -> Result<f32, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    match it.next() {
        Some(PathToken::Number(v)) => Ok(v),
        _ => Err(SvgImportError::InvalidPathData),
    }
}

/// Parse and emit any buffered number text, then clear the buffer.
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

/// Report whether a character separates path tokens.
const fn is_separator(ch: char) -> bool { ch.is_ascii_whitespace() || ch == ',' }

// This importer currently supports the absolute command subset emitted by
// Gauss export (`M`, `L`, `C`, `Z`) and rejects relative commands.
/// Report whether a character is a supported path command letter.
const fn is_command(ch: char) -> bool { matches!(ch, 'M' | 'L' | 'C' | 'Z') }

/// Report whether a character may appear within a number token.
const fn is_number_char(ch: char) -> bool { ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.') }

/// Report whether a sign starts a new number, as in `10-5`, rather than
/// continuing the buffered one.
const fn is_implicit_number_separator(ch: char, number_buf: &str) -> bool {
    !number_buf.is_empty() && matches!(ch, '-' | '+')
}

/// Split path-data text into command and number tokens.
///
/// Numbers may be separated by whitespace, commas, or a leading sign.
fn tokenize_path_data(d: &str) -> Result<Vec<PathToken>, SvgImportError> {
    let mut tokens = Vec::new();
    let mut number_buf = String::new();

    for ch in d.chars() {
        if is_separator(ch) {
            flush_number(&mut number_buf, &mut tokens)?;
            continue;
        }

        if is_command(ch) {
            flush_number(&mut number_buf, &mut tokens)?;
            tokens.push(PathToken::Command(ch));
            continue;
        }

        if is_implicit_number_separator(ch, number_buf.as_str()) {
            flush_number(&mut number_buf, &mut tokens)?;
            number_buf.push(ch);
            continue;
        }

        if is_number_char(ch) {
            number_buf.push(ch);
            continue;
        }

        return Err(SvgImportError::InvalidPathData);
    }

    flush_number(&mut number_buf, &mut tokens)?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    //! Tests for path-data tokenization and parsing.

    use rstest::rstest;

    use super::parse_path_data;

    #[rstest]
    fn parses_plus_as_number_separator() {
        let parsed = parse_path_data("M 0 0 L 10+5").expect("path data should parse");
        assert_eq!(parsed.anchors.len(), 2);
        let Some(anchor) = parsed.anchors.get(1) else {
            panic!("parsed path should include the line endpoint anchor");
        };
        assert_eq!(anchor.pos.x.to_bits(), 10.0_f32.to_bits());
        assert_eq!(anchor.pos.y.to_bits(), 5.0_f32.to_bits());
    }
}
