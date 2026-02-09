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

fn parse_tag_attributes(tag: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
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

    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with('>') || rest.starts_with("/>") {
            break;
        }

        let Some(eq_index) = rest.find('=') else {
            break;
        };

        let name = rest.get(..eq_index).unwrap_or_default().trim();
        rest = rest.get((eq_index + 1)..).unwrap_or_default().trim_start();
        let Some(quote) = rest.chars().next() else {
            break;
        };
        if quote != '"' && quote != '\'' {
            break;
        }

        rest = rest.get(quote.len_utf8()..).unwrap_or_default();
        let Some(end_quote) = rest.find(quote) else {
            break;
        };

        let value = rest.get(..end_quote).unwrap_or_default();
        if !name.is_empty() {
            attributes.push((name.to_owned(), value.to_owned()));
        }

        rest = rest
            .get((end_quote + quote.len_utf8())..)
            .unwrap_or_default();
    }

    attributes
}
