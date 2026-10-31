//! Parses inline YAML collections while preserving scalar spelling and rejecting ambiguity.

use std::collections::BTreeMap;

use super::super::Value;
use super::Scanner;

/// Supported YAML double-quoted escape codes and their decoded characters.
const DOUBLE_QUOTED_ESCAPES: &[(char, char)] = &[
    ('0', '\0'),
    ('a', '\u{7}'),
    ('b', '\u{8}'),
    ('t', '\t'),
    ('n', '\n'),
    ('v', '\u{b}'),
    ('f', '\u{c}'),
    ('r', '\r'),
    ('e', '\u{1b}'),
    (' ', ' '),
    ('"', '"'),
    ('/', '/'),
    ('\\', '\\'),
];

/// Parses one quoted or plain scalar and rejects unsupported YAML markers.
pub(in super::super) fn parse_inline(source: &str, number: usize) -> Result<Value, String> {
    FlowReader::new(source, number).document()
}

/// Parses inline collections without YAML 1.1 boolean coercion.
struct FlowReader<'a> {
    /// Inline source text from one physical line.
    source: &'a str,
    /// Byte offset at a UTF-8 character boundary.
    cursor: usize,
    /// Physical source line used in errors.
    number: usize,
}

impl<'a> FlowReader<'a> {
    /// Creates a cursor over one inline YAML value.
    const fn new(source: &'a str, number: usize) -> Self {
        Self {
            source,
            cursor: 0,
            number,
        }
    }

    /// Parses one inline value and requires all non-whitespace input to be consumed.
    fn document(mut self) -> Result<Value, String> {
        let value = self.value()?;
        self.space();
        if self.cursor != self.source.len() {
            return self.fail("unexpected text after inline value");
        }
        Ok(value)
    }

    /// Parses one flow collection or scalar according to its first character.
    fn value(&mut self) -> Result<Value, String> {
        self.space();
        match self.peek() {
            Some('[') => self.sequence(),
            Some('{') => self.mapping(),
            Some('\'' | '"') => self.quoted(),
            Some(_) => self.plain(&[',', ']', '}']),
            None => Ok(Value::Empty),
        }
    }

    /// Parses a flow sequence and rejects missing separators or trailing commas.
    fn sequence(&mut self) -> Result<Value, String> {
        self.take('[')?;
        let mut items = Vec::new();
        self.space();
        if self.peek() == Some(']') {
            self.cursor += 1;
            return Ok(Value::Sequence(items));
        }
        loop {
            items.push(self.value()?);
            self.space();
            match self.peek() {
                Some(',') => {
                    self.cursor += 1;
                    self.space();
                }
                Some(']') => {
                    self.cursor += 1;
                    return Ok(Value::Sequence(items));
                }
                _ => return self.fail("expected `,` or `]` in flow sequence"),
            }
            if self.peek() == Some(']') {
                return self.fail("trailing flow-sequence comma is unsupported");
            }
        }
    }

    /// Parses a flow mapping and detects duplicate decoded keys.
    fn mapping(&mut self) -> Result<Value, String> {
        self.take('{')?;
        let mut entries = BTreeMap::new();
        self.space();
        if self.peek() == Some('}') {
            self.cursor += 1;
            return Ok(Value::Mapping(entries));
        }
        loop {
            self.mapping_entry(&mut entries)?;
            if self.mapping_delimiter()? {
                return Ok(Value::Mapping(entries));
            }
        }
    }

    /// Parses one decoded key/value pair into the mapping being built.
    /// For example, `{city: York}` adds the key `city` with value `York`.
    fn mapping_entry(&mut self, entries: &mut BTreeMap<String, Value>) -> Result<(), String> {
        let key = self.key()?;
        if key == "<<" {
            return self.fail("YAML merge keys are unsupported");
        }
        if entries.contains_key(&key) {
            return self.fail(&format!("duplicate decoded mapping key `{key}`"));
        }
        self.space();
        self.take(':')?;
        self.space();
        let value = if matches!(self.peek(), Some(',' | '}')) {
            Value::Empty
        } else {
            self.value()?
        };
        entries.insert(key, value);
        Ok(())
    }

    /// Consumes a mapping comma or closing brace, retaining trailing-comma errors.
    /// For example, after `city: York}`, it consumes `}` and returns `true`.
    fn mapping_delimiter(&mut self) -> Result<bool, String> {
        self.space();
        match self.peek() {
            Some(',') => {
                self.cursor += 1;
                self.space();
                if self.peek() == Some('}') {
                    self.fail("trailing flow-mapping comma is unsupported")
                } else {
                    Ok(false)
                }
            }
            Some('}') => {
                self.cursor += 1;
                Ok(true)
            }
            _ => self.fail("expected `,` or `}` in flow mapping"),
        }
    }

    /// Decodes a scalar mapping key and preserves boolean-like text as a string.
    fn key(&mut self) -> Result<String, String> {
        self.space();
        if matches!(self.peek(), Some('\'' | '"')) {
            self.quoted_key()
        } else {
            self.plain_key()
        }
    }

    /// Accepts a decoded quoted key only when it is a non-empty scalar.
    /// For example, `'on'` remains a text key rather than a boolean.
    fn quoted_key(&mut self) -> Result<String, String> {
        match self.quoted()? {
            Value::Scalar(key) if !key.is_empty() => Ok(key),
            _ => self.fail("mapping keys must be non-empty scalars"),
        }
    }

    /// Acquires a plain key span, stopping before its colon or first forbidden flow token.
    /// For example, `city: York` leaves the cursor at the colon after `city`.
    fn plain_key(&mut self) -> Result<String, String> {
        let start = self.cursor;
        while let Some(character) = self.peek() {
            if character == ':' {
                break;
            }
            if matches!(character, '[' | ']' | '{' | '}' | ',') {
                return self.fail("unsupported flow mapping key");
            }
            self.cursor += character.len_utf8();
        }
        self.plain_key_value(start)
    }

    /// Validates the acquired span before allocating its plain scalar value.
    /// For example, a leading alias marker fails after the same checked slice lookup.
    fn plain_key_value(&self, start: usize) -> Result<String, String> {
        let source = self
            .source
            .get(start..self.cursor)
            .ok_or_else(|| format!("line {}: invalid mapping key boundary", self.number))?
            .trim();
        if source.is_empty() || matches!(source.chars().next(), Some('&' | '*' | '!')) {
            return self.fail("mapping keys must be non-empty plain scalars");
        }
        Ok(source.to_owned())
    }

    /// Reports whether the consumed character begins a doubled YAML single quote.
    /// For example, the first doubled quote in `'can''t'` returns `true`.
    fn is_doubled_single_quote(&self, quote: char, character: char) -> bool {
        character == quote && quote == '\'' && self.peek() == Some('\'')
    }

    /// Decodes a single- or double-quoted scalar with a deliberately small escape set.
    fn quoted(&mut self) -> Result<Value, String> {
        let quote = self
            .peek()
            .ok_or_else(|| self.message("expected quoted scalar"))?;
        self.cursor += quote.len_utf8();
        let mut output = String::new();
        while let Some(character) = self.peek() {
            self.cursor += character.len_utf8();
            if self.is_doubled_single_quote(quote, character) {
                self.cursor += 1;
                output.push('\'');
            } else if character == quote {
                return Ok(Value::Scalar(output));
            } else if quote == '"' && character == '\\' {
                self.escaped(&mut output)?;
            } else {
                output.push(character);
            }
        }
        self.fail("unterminated quoted scalar")
    }

    /// Decodes one supported YAML double-quoted escape.
    fn escaped(&mut self, output: &mut String) -> Result<(), String> {
        let escaped = self
            .peek()
            .ok_or_else(|| self.message("incomplete quoted escape"))?;
        self.cursor += escaped.len_utf8();
        let decoded_value = DOUBLE_QUOTED_ESCAPES
            .iter()
            .find_map(|(code, value)| (*code == escaped).then_some(*value));
        let Some(decoded) = decoded_value else {
            let diagnostic = if matches!(escaped, 'N' | '_' | 'L' | 'P' | 'x' | 'u' | 'U') {
                "Unicode and line-break escapes are unsupported"
            } else {
                "unknown double-quoted escape"
            };
            return self.fail(diagnostic);
        };
        output.push(decoded);
        Ok(())
    }

    /// Reads a plain scalar until an inline collection separator.
    fn plain(&mut self, delimiters: &[char]) -> Result<Value, String> {
        let start = self.cursor;
        let mut scanner = Scanner::default();
        while let Some(character) = self.peek() {
            if self.scan_plain_character(character, delimiters, &mut scanner)? {
                break;
            }
        }
        if scanner.has_unclosed_syntax() {
            return self.fail("unbalanced quote or flow delimiter");
        }
        let source = self
            .source
            .get(start..self.cursor)
            .ok_or_else(|| format!("line {}: invalid scalar boundary", self.number))?
            .trim();
        if matches!(source.chars().next(), Some('&' | '*' | '!')) {
            return self.fail("anchors, aliases, and tags are unsupported");
        }
        if matches!(source, "~" | "null" | "Null" | "NULL") {
            Ok(Value::Empty)
        } else if source.is_empty() {
            self.fail("empty plain scalar")
        } else {
            Ok(Value::Scalar(source.to_owned()))
        }
    }

    /// Advances one plain-scalar character or reports that it is an outer delimiter.
    fn scan_plain_character(
        &mut self,
        character: char,
        delimiters: &[char],
        scanner: &mut Scanner,
    ) -> Result<bool, String> {
        let next = self
            .source
            .get(self.cursor + character.len_utf8()..)
            .and_then(|rest| rest.chars().next());
        if scanner.consume_quoted(character, next) {
            self.cursor += character.len_utf8();
            return Ok(false);
        }
        match character {
            '[' => scanner.nesting.push(']'),
            '{' => scanner.nesting.push('}'),
            ']' | '}' if scanner.nesting.last() == Some(&character) => {
                let length = scanner.nesting.len().saturating_sub(1);
                scanner.nesting.truncate(length);
            }
            ']' | '}' | ',' if scanner.nesting.is_empty() && delimiters.contains(&character) => {
                return Ok(true);
            }
            ']' | '}' => return self.fail("unbalanced flow delimiter"),
            _ => {}
        }
        self.cursor += character.len_utf8();
        scanner.previous = Some(character);
        Ok(false)
    }

    /// Consumes one exact delimiter after optional whitespace.
    fn take(&mut self, expected: char) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.cursor += expected.len_utf8();
            Ok(())
        } else {
            self.fail(&format!("expected `{expected}`"))
        }
    }

    /// Skips inline whitespace before a value or collection delimiter.
    fn space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            if let Some(character) = self.peek() {
                self.cursor += character.len_utf8();
            }
        }
    }

    /// Returns the next UTF-8 character without advancing the byte cursor.
    fn peek(&self) -> Option<char> {
        self.source.get(self.cursor..)?.chars().next()
    }

    /// Builds a line-qualified parser error.
    fn message(&self, message: &str) -> String {
        format!("line {}: {message}", self.number)
    }

    /// Returns one line-qualified failure for a flow parsing branch.
    fn fail<T>(&self, message: &str) -> Result<T, String> {
        Err(self.message(message))
    }
}
