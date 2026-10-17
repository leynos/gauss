//! Tokenizes bounded shell rows for Make invocation classification.

/// Shell tokens needed to preserve command order and failure delimiters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Token {
    /// A shell word after quote removal.
    Word(String),
    /// A command that runs only when the preceding command succeeds.
    And,
    /// A command whose status may hide an earlier failure.
    Or,
    /// A pipe that may hide a command's status.
    Pipe,
    /// A semicolon separating independently executed commands.
    Semi,
    /// A command started asynchronously.
    Background,
    /// A redirection kept separate from executable arguments.
    Redirect(String),
}

/// Carries quote and escape state while one bounded shell command is tokenized.
#[derive(Default)]
struct Tokenizer {
    /// Completed shell words and control delimiters.
    tokens: Vec<Token>,
    /// Word currently being assembled after quote removal.
    word: String,
    /// Active single or double quote, when present.
    quote: Option<char>,
    /// Whether any portion of the current word was quoted.
    word_quoted: bool,
    /// Whether the active quote has supplied content.
    quote_content: bool,
    /// Whether the preceding character escaped the current one.
    escaped: bool,
}

/// Finds a shell suffix at a byte cursor without changing tokenizer state.
/// For example, source `"cmd"` at cursor `0` returns `"cmd"`.
fn shell_suffix(source: &str, cursor: usize) -> Result<&str, String> {
    source
        .get(cursor..)
        .ok_or_else(|| "invalid shell cursor".to_owned())
}

/// Reads the next character from a shell suffix without advancing its cursor.
/// For example, suffix `"cmd"` yields the first character `c`.
fn first_shell_character(suffix: &str) -> Result<char, String> {
    suffix
        .chars()
        .next()
        .ok_or_else(|| "empty shell suffix".to_owned())
}

/// Tokenizes a bounded POSIX-shell command while retaining all control delimiters.
pub(super) fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokenizer = Tokenizer::default();
    let mut cursor = 0;
    while cursor < source.len() {
        let rest = shell_suffix(source, cursor)?;
        if tokenizer.consume_operator(rest, &mut cursor)? {
            continue;
        }
        let character = first_shell_character(rest)?;
        cursor += character.len_utf8();
        tokenizer.consume_character(character)?;
    }
    tokenizer.finish()
}

impl Tokenizer {
    /// Consumes one unquoted operator and flushes any word before it.
    /// For example, `&&` becomes a delimiter between two shell commands.
    fn consume_operator(&mut self, source: &str, cursor: &mut usize) -> Result<bool, String> {
        let Some((token, length)) = operator(source) else {
            return Ok(false);
        };
        if self.quote.is_some() {
            return Ok(false);
        }
        self.flush_word()?;
        self.tokens.push(token);
        *cursor += length;
        Ok(true)
    }

    /// Reports whether opening a quote would create an unsupported mixed fragment.
    /// For example, `command` is rejected while an assignment prefix `NAME=` is allowed.
    fn has_unsupported_quote_prefix(&self) -> bool {
        self.word_quoted || (!self.word.is_empty() && !assignment_prefix(&self.word))
    }

    /// Applies one character to quote, escape, or current-word state.
    /// For example, whitespace flushes a word only when no quote is active.
    fn consume_character(&mut self, character: char) -> Result<(), String> {
        if self.consume_escape_or_start(character) || self.consume_quote(character)? {
            return Ok(());
        }
        if self.consume_unquoted_separator(character)? {
            return Ok(());
        }
        self.word.push(character);
        self.quote_content |= self.quote.is_some();
        Ok(())
    }

    /// Applies a pending escape or records the next escape marker.
    /// For example, an escaped space is appended to the current word without splitting it.
    fn consume_escape_or_start(&mut self, character: char) -> bool {
        if self.escaped {
            self.word.push(character);
            self.quote_content |= self.quote.is_some();
            self.escaped = false;
            return true;
        }
        if character == '\\' && self.quote != Some('\'') {
            self.escaped = true;
            return true;
        }
        false
    }

    /// Opens or closes a quote while refusing empty and mixed quoted fragments.
    /// For example, `"word"` closes cleanly and `""` returns an error.
    fn consume_quote(&mut self, character: char) -> Result<bool, String> {
        if self.quote == Some(character) {
            if !self.quote_content {
                return Err("empty quoted shell word is unsupported".to_owned());
            }
            self.quote = None;
            return Ok(true);
        }
        if self.quote.is_none() && matches!(character, '\'' | '"') {
            if self.has_unsupported_quote_prefix() {
                return Err("mixed quoted shell fragments are unsupported".to_owned());
            }
            self.quote = Some(character);
            self.word_quoted = true;
            self.quote_content = false;
            return Ok(true);
        }
        Ok(false)
    }

    /// Flushes whitespace only outside quotes and rejects text after a closed quoted fragment.
    /// For example, `"word" tail` is split while `"word"tail` is rejected.
    fn consume_unquoted_separator(&mut self, character: char) -> Result<bool, String> {
        if self.quote.is_some() {
            return Ok(false);
        }
        if character.is_whitespace() {
            self.flush_word()?;
            return Ok(true);
        }
        if self.word_quoted {
            return Err("mixed quoted shell fragments are unsupported".to_owned());
        }
        Ok(false)
    }

    /// Flushes the final word or rejects an unfinished quote or escape.
    /// For example, a trailing backslash returns an unbalanced-input error.
    fn finish(mut self) -> Result<Vec<Token>, String> {
        if self.escaped || self.quote.is_some() {
            return Err("unbalanced shell quote or escape".to_owned());
        }
        self.flush_word()?;
        Ok(self.tokens)
    }

    /// Adds a shell word only when quote removal produced content.
    /// For example, a nonempty buffer becomes one `Word` token.
    fn flush_word(&mut self) -> Result<(), String> {
        if self.word.is_empty() {
            if self.word_quoted {
                return Err("empty quoted shell word is unsupported".to_owned());
            }
        } else {
            if self.word_quoted && matches!(self.word.as_str(), "if" | "then" | "else" | "fi") {
                return Err("quoted shell control word is unsupported".to_owned());
            }
            self.word_quoted = false;
            self.tokens
                .push(Token::Word(std::mem::take(&mut self.word)));
        }
        Ok(())
    }
}

/// Recognizes a shell word prefix that names a variable assignment.
fn assignment_prefix(word: &str) -> bool {
    word.strip_suffix('=').is_some_and(valid_variable_name)
}

/// Restricts assignment names to literal Make variable identifiers.
pub(super) fn valid_variable_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first == '.' || first.is_ascii_alphabetic())
        && characters.all(|character| {
            character == '_'
                || character == '.'
                || character == '-'
                || character.is_ascii_alphanumeric()
        })
}

/// Joins shell backslash continuations without discarding separators or empty results.
pub(super) fn shell_lines(source: &str) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    let mut pending = String::new();
    for line in source.lines() {
        let current = line.trim();
        if current.is_empty() {
            continue;
        }
        let continued = trailing_backslash(current);
        let mut continued_text = current.to_owned();
        if continued {
            let _ = continued_text.pop();
        }
        let fragment = if continued {
            continued_text.trim_end()
        } else {
            current
        };
        if !pending.is_empty() {
            pending.push(' ');
        }
        pending.push_str(fragment);
        if !continued {
            lines.push(std::mem::take(&mut pending));
        }
    }
    if pending.is_empty() {
        Ok(lines)
    } else {
        Err("unterminated shell line continuation".to_owned())
    }
}

/// Removes comments while respecting the single-character Make escape before `#`.
pub(super) fn without_comment(line: &str) -> &str {
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if character == '#' && !escaped {
            return line.get(..index).unwrap_or("");
        }
        escaped = character == '\\' && !escaped;
        if character != '\\' {
            escaped = false;
        }
    }
    line
}

/// Locates an unescaped rule colon outside simple parenthesized Make references.
pub(super) fn rule_separator(line: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (index, character) in line.char_indices() {
        match character {
            '$' if line
                .get(index + 1..)
                .is_some_and(|tail| tail.starts_with('(')) =>
            {
                depth += 1;
            }
            ')' if depth > 0 => depth -= 1,
            ':' if depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

/// Detects an odd trailing run of backslashes, which continues the logical line.
pub(super) fn trailing_backslash(text: &str) -> bool {
    // Odd parity leaves one continuation marker after paired literal backslashes.
    text.chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count()
        & 1
        == 1
}

/// Recognizes operators longest-first so redirection text cannot become a word.
fn operator(source: &str) -> Option<(Token, usize)> {
    [
        ("2>&1", Token::Redirect("2>&1".to_owned())),
        (">/dev/null", Token::Redirect(">/dev/null".to_owned())),
        ("&&", Token::And),
        ("||", Token::Or),
        (">>", Token::Redirect(">>".to_owned())),
        (";", Token::Semi),
        ("|", Token::Pipe),
        ("&", Token::Background),
        ("<", Token::Redirect("<".to_owned())),
        (">", Token::Redirect(">".to_owned())),
    ]
    .into_iter()
    .find_map(|(text, token)| source.starts_with(text).then_some((token, text.len())))
}
