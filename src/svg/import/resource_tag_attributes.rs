//! Helpers for parsing attribute lists from SVG opening tags.

pub(super) fn opening_tag(block: &str) -> Option<&str> {
    let open_end = block.find('>')?;
    block.get(..=open_end)
}

pub(super) fn collect_extra_attributes(
    tag: &str,
    excluded_names: &[&str],
) -> Vec<(String, String)> {
    parse_tag_attributes(tag)
        .into_iter()
        .filter(|(name, _)| !excluded_names.contains(&name.as_str()))
        .collect()
}

fn skip_to_attributes(tag: &str) -> &str {
    let mut rest = tag.trim();
    if let Some(start) = rest.find('<') {
        rest = rest.get((start + 1)..).unwrap_or_default();
    }
    while let Some(ch) = rest.chars().next() {
        if ch.is_whitespace() {
            break;
        }
        rest = rest.get(ch.len_utf8()..).unwrap_or_default();
    }
    rest
}

fn try_parse_next_attribute(input: &str) -> Option<(String, String, &str)> {
    let rest = input.trim_start();
    if rest.is_empty() {
        return None;
    }
    if rest.starts_with('>') {
        return None;
    }
    if rest.starts_with("/>") {
        return None;
    }

    let eq_index = rest.find('=')?;
    let name = rest.get(..eq_index)?.trim();
    if name.is_empty() {
        return None;
    }

    let value_start = rest.get((eq_index + 1)..)?.trim_start();
    let quote = value_start.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let value_input = value_start.get(quote.len_utf8()..)?;
    let end_quote = value_input.find(quote)?;
    let value = value_input.get(..end_quote)?;
    let remaining = value_input.get((end_quote + quote.len_utf8())..)?;

    Some((name.to_owned(), value.to_owned(), remaining))
}

fn parse_tag_attributes(tag: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    let mut rest = skip_to_attributes(tag);

    while let Some((name, value, remaining)) = try_parse_next_attribute(rest) {
        attributes.push((name, value));
        rest = remaining;
    }

    attributes
}
