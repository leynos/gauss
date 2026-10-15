//! SVG path-data parsing helpers.

use crate::model::{Anchor, PathGeom, SegmentKind, Vec2};

use super::SvgImportError;

#[derive(Clone, Debug, PartialEq)]
/// A token produced while scanning the supported path-data subset.
///
/// Keeping commands separate from parsed numbers lets the parser reject a
/// number where a command is required.
enum PathToken {
    /// One recognized uppercase absolute command: `M`, `L`, `C`, or `Z`.
    Command(char),
    /// One coordinate number parsed from the path-data text.
    Number(f32),
}

/// Parses the absolute `M`, `L`, `C`, and `Z` path-data subset.
///
/// ASCII whitespace and commas separate values; a sign may separate adjacent
/// numbers, so `M 0 0 L 10+5` ends at `(10, 5)`. Exponent notation is not
/// tokenized. Empty input returns empty geometry; malformed text or token
/// order returns `InvalidPathData`.
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

/// Consumes two number tokens as the x and y coordinates of a point.
///
/// For example, `Number(10), Number(5)` produces `Vec2::new(10.0, 5.0)`;
/// a missing or non-number token returns `InvalidPathData`.
fn next_vec2<I>(it: &mut I) -> Result<Vec2, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    let x = next_number(it)?;
    let y = next_number(it)?;
    Ok(Vec2::new(x, y))
}

/// Appends an anchor at the point formed by the next two number tokens.
///
/// For example, `Number(4), Number(2)` adds an anchor at `(4, 2)` without
/// adding a segment.
fn parse_move_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<(), SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    geom.anchors.push(Anchor::new(next_vec2(it)?));
    Ok(())
}

/// Adds a line segment and an endpoint from the next coordinate pair.
///
/// For example, `Number(4), Number(2)` appends an anchor at `(4, 2)` and
/// returns `SegmentKind::Line`.
fn parse_line_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<SegmentKind, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    geom.segments.push(SegmentKind::Line);
    geom.anchors.push(Anchor::new(next_vec2(it)?));
    Ok(SegmentKind::Line)
}

/// Adds a cubic segment from its two handles, then its endpoint.
///
/// After an anchor at `(0, 0)`, `1 2 3 4 5 6` stores `(1, 2)` on that anchor,
/// then adds an endpoint at `(5, 6)` with incoming handle `(3, 4)`. A preceding
/// anchor is required; otherwise this returns `InvalidPathData`.
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

/// Marks the geometry closed and selects the closing segment kind.
///
/// `Some(SegmentKind::Cubic)` keeps a cubic closing segment; `None` defaults
/// to `SegmentKind::Line`.
fn close_path(geom: &mut PathGeom, last_segment: Option<SegmentKind>) {
    geom.closed = true;
    geom.closing_segment = last_segment.unwrap_or(SegmentKind::Line);
}

/// Consumes the next number token as an `f32`.
///
/// `Number(5.0)` yields `5.0`; a missing token or command token returns
/// `InvalidPathData`.
fn next_number<I>(it: &mut I) -> Result<f32, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    match it.next() {
        Some(PathToken::Number(v)) => Ok(v),
        _ => Err(SvgImportError::InvalidPathData),
    }
}

/// Parse a non-empty number buffer into a token, clearing it only on success.
///
/// For example, `"10.5"` appends `Number(10.5)` and becomes empty; invalid
/// numeric text returns `InvalidPathData`. An empty buffer is left untouched.
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

/// Recognizes ASCII whitespace and commas between path-data values.
///
/// For example, `' '` and `','` are separators, while `';'` is not.
const fn is_separator(ch: char) -> bool {
    ch.is_ascii_whitespace() || ch == ','
}

// This importer currently supports the absolute command subset emitted by
// Gauss export (`M`, `L`, `C`, `Z`) and rejects relative commands.
/// Recognizes the supported uppercase absolute command letters.
///
/// `M` is recognized, while lowercase `m` is not.
const fn is_command(ch: char) -> bool {
    matches!(ch, 'M' | 'L' | 'C' | 'Z')
}

/// Recognizes characters admitted to a numeric buffer before `f32` parsing.
///
/// ASCII digits, signs, and `.` are accepted; `e` is not, so exponent
/// notation is outside this tokenizer's number syntax.
const fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.')
}

/// Treats a sign as a boundary when it follows buffered number text.
///
/// In `10+5`, `+` flushes `10` and begins the next number; with an empty
/// buffer, a sign starts a number instead.
const fn is_implicit_number_separator(ch: char, number_buf: &str) -> bool {
    !number_buf.is_empty() && matches!(ch, '-' | '+')
}

/// Splits path text into recognized command and parsed number tokens.
///
/// For example, `M 0 0 L 10+5` yields `M`, `0`, `0`, `L`, `10`, and `5`.
/// Empty input yields no tokens; malformed numeric text or unrecognized
/// characters return `InvalidPathData`.
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

    use super::parse_path_data;
    use rstest::rstest;

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
