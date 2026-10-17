//! Provides line and inline lexical scanning for the strict workflow YAML subset.

/// Keeps inline parsing beside its flow-specific cursor while retaining the parent entry point.
#[path = "ci_reachability_lines_flow.rs"]
mod flow;

/// Retains the inline parser entry point used by the parent YAML reader.
pub(super) use flow::parse_inline;

/// Removes a YAML comment while preserving hashes inside quoted values and tokens.
pub(super) fn without_comment(line: &str) -> Result<&str, String> {
    let mut scanner = Scanner::default();
    for (index, character) in line.char_indices() {
        let next = line
            .get(index + character.len_utf8()..)
            .and_then(|rest| rest.chars().next());
        if scanner.consume_quoted(character, next) {
            continue;
        }
        if character == '#' && scanner.previous.is_none_or(char::is_whitespace) {
            return line
                .get(..index)
                .ok_or_else(|| "invalid comment boundary".to_owned());
        }
        scanner.previous = Some(character);
    }
    Ok(line)
}

/// Holds only quote and delimiter state shared by the line-level lexical checks.
#[derive(Default)]
struct Scanner {
    /// Active YAML quote character, if the cursor is inside a quoted scalar.
    quote: Option<char>,
    /// Whether a quoted scalar has an escape awaiting its next character.
    escaped: bool,
    /// Previous source character used to recognize the start of a quoted token.
    previous: Option<char>,
    /// Expected closing delimiters for nested flow collections.
    nesting: Vec<char>,
}

impl Scanner {
    /// Reports whether scanning ended inside a quote, escape, or flow collection.
    /// For example, an unmatched `[` leaves this query true.
    const fn has_unclosed_syntax(&self) -> bool {
        self.quote.is_some() || self.escaped || !self.nesting.is_empty()
    }

    /// Identifies quote-specific escape forms before their next character is consumed.
    /// For example, `\\"` inside a double-quoted scalar starts an escape.
    const fn starts_quoted_escape(&self, character: char, next: Option<char>) -> bool {
        matches!(
            (self.quote, character, next),
            (Some('"'), '\\', _) | (Some('\''), '\'', Some('\''))
        )
    }

    /// Consumes quote syntax and reports whether the character belongs to quoted text.
    fn consume_quoted(&mut self, character: char, next: Option<char>) -> bool {
        if self.escaped {
            self.escaped = false;
        } else if self.starts_quoted_escape(character, next) {
            self.escaped = true;
        } else if let Some(quote) = self.quote {
            if character == quote {
                self.quote = None;
            }
        } else if matches!(character, '\'' | '"')
            && self.previous.is_none_or(|previous| {
                previous.is_whitespace() || matches!(previous, '[' | '{' | ',' | ':')
            })
        {
            self.quote = Some(character);
        } else {
            return false;
        }
        self.previous = Some(character);
        true
    }
    /// Applies a flow delimiter before an unquoted mapping colon is considered.
    /// For example, `]` with no open collection reports an unbalanced delimiter.
    fn consume_flow_delimiter(&mut self, character: char) -> Result<(), String> {
        match character {
            '[' => self.nesting.push(']'),
            '{' => self.nesting.push('}'),
            ']' | '}' if self.nesting.last() == Some(&character) => {
                let length = self.nesting.len().saturating_sub(1);
                self.nesting.truncate(length);
            }
            ']' | '}' if self.nesting.is_empty() => {
                return Err("unbalanced flow delimiter".to_owned());
            }
            ']' | '}' => return Err("mismatched flow delimiter".to_owned()),
            _ => {}
        }
        Ok(())
    }

    /// Identifies a colon outside flow collections without inspecting its suffix yet.
    /// For example, the colon in `[city: York]` is not a block mapping separator.
    const fn is_mapping_colon(&self, character: char) -> bool {
        character == ':' && self.nesting.is_empty()
    }
}

/// Finds a block mapping separator without treating quoted or URL colons as separators.
pub(super) fn mapping_separator(text: &str) -> Result<Option<usize>, String> {
    let mut scanner = Scanner::default();
    for (index, character) in text.char_indices() {
        let next = text
            .get(index + character.len_utf8()..)
            .and_then(|rest| rest.chars().next());
        if scanner.consume_quoted(character, next) {
            continue;
        }
        scanner.consume_flow_delimiter(character)?;
        if scanner.is_mapping_colon(character)
            && mapping_value_follows(text, index + character.len_utf8())?
        {
            return Ok(Some(index));
        }
        scanner.previous = Some(character);
    }
    if scanner.has_unclosed_syntax() {
        Err("unbalanced quote or flow delimiter".to_owned())
    } else {
        Ok(None)
    }
}

/// Checks the suffix boundary required after a potential block mapping colon.
/// For example, `city: York` passes while `https://example.test` does not.
fn mapping_value_follows(text: &str, after: usize) -> Result<bool, String> {
    let rest = text
        .get(after..)
        .ok_or_else(|| "invalid mapping separator boundary".to_owned())?;
    Ok(rest.is_empty() || rest.chars().next().is_some_and(char::is_whitespace))
}
