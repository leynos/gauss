//! SVG path-data parsing helpers.

use crate::model::{Anchor, PathGeom, SegmentKind, Vec2};

use super::SvgImportError;

#[derive(Clone, Debug, PartialEq)]
enum PathToken {
    Command(char),
    Number(f32),
}

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

fn next_vec2<I>(it: &mut I) -> Result<Vec2, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    let x = next_number(it)?;
    let y = next_number(it)?;
    Ok(Vec2::new(x, y))
}

fn parse_move_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<(), SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    geom.anchors.push(Anchor::new(next_vec2(it)?));
    Ok(())
}

fn parse_line_command<I>(it: &mut I, geom: &mut PathGeom) -> Result<SegmentKind, SvgImportError>
where
    I: Iterator<Item = PathToken>,
{
    geom.segments.push(SegmentKind::Line);
    geom.anchors.push(Anchor::new(next_vec2(it)?));
    Ok(SegmentKind::Line)
}

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

fn close_path(geom: &mut PathGeom, last_segment: Option<SegmentKind>) {
    geom.closed = true;
    geom.closing_segment = last_segment.unwrap_or(SegmentKind::Line);
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

const fn is_separator(ch: char) -> bool {
    ch.is_ascii_whitespace() || ch == ','
}

const fn is_command(ch: char) -> bool {
    matches!(ch, 'M' | 'L' | 'C' | 'Z')
}

const fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.')
}

const fn is_negative_number_start(ch: char, number_buf: &str) -> bool {
    ch == '-' && !number_buf.is_empty()
}

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

        if is_negative_number_start(ch, number_buf.as_str()) {
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
