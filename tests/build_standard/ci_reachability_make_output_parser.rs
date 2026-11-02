//! Parses bounded shell command rows without losing status delimiters.

use super::{Invocation, Route, make_syntax::Token, make_tools};

/// One command and its separator, retained until the full sequence has been inspected.
#[derive(Clone, Debug)]
pub(super) struct Resolved {
    /// Complete route classification for this executable.
    pub(super) route: Route,
    /// Failure relationship to the next command.
    pub(super) follow: Follow,
    /// Original words and redirects used by narrow control-flow checks.
    pub(super) invocation: Invocation,
}

/// Delimiters that either preserve or can hide a command's status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Follow {
    /// The invocation is the final command in its shell sequence.
    End,
    /// The following command runs only when this invocation succeeds.
    And,
    /// A reviewed semicolon separates the fallback notice from Cargo.
    Semi,
    /// The following command may be skipped when this invocation succeeds.
    Or,
    /// The invocation's status may be replaced by a pipeline's final command.
    Pipe,
    /// The invocation runs asynchronously and does not determine shell status.
    Background,
}

/// Mutable parsing state avoids long argument lists while retaining every command row.
struct CommandParser<F> {
    /// Whether private probe names have already been authorized by the caller.
    controlled: bool,
    /// Resolver for each complete invocation.
    resolver: F,
    /// Commands parsed so far.
    commands: Vec<Resolved>,
    /// Words and metadata for the command currently being built.
    pending: Pending,
}

/// Unfinished invocation, including a redirection whose destination is still required.
#[derive(Default)]
struct Pending {
    /// Executable and ordinary arguments.
    words: Vec<String>,
    /// Leading shell environment assignments.
    assignments: Vec<String>,
    /// Validated redirection tokens with their destinations.
    redirects: Vec<String>,
    /// Redirection awaiting one destination word.
    redirect: Option<String>,
}

impl<F> CommandParser<F>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    /// Parses every token and resolves every completed invocation in order.
    fn parse(mut self, tokens: &[Token]) -> Result<Vec<Resolved>, String> {
        for token in tokens {
            match token {
                Token::Word(word) => self.push_word(word)?,
                Token::Redirect(value) => self.push_redirect(value)?,
                Token::And => self.push(Follow::And)?,
                Token::Semi => self.push(Follow::Semi)?,
                Token::Or => self.push(Follow::Or)?,
                Token::Pipe => self.push(Follow::Pipe)?,
                Token::Background => self.push(Follow::Background)?,
            }
        }
        if self.pending.redirect.is_some() {
            return Err("redirection has no destination".to_owned());
        }
        self.push(Follow::End)?;
        Ok(self.commands)
    }

    /// Adds one word while preserving pending redirects, assignments, and literal exceptions.
    /// For example, a word after `>>` becomes the redirect destination before command argv.
    fn push_word(&mut self, word: &str) -> Result<(), String> {
        if let Some(redirect) = self.pending.redirect.take() {
            self.pending.redirects.push(format!("{redirect}{word}"));
            return Ok(());
        }
        if self.pending.words.is_empty() && is_assignment(word) {
            self.pending.assignments.push(word.to_owned());
            return Ok(());
        }
        if is_literal_echo_argument(&self.pending.words, word) {
            self.pending.words.push(word.to_owned());
            return Ok(());
        }
        if word.contains('$') || word.contains('`') {
            return Err("unreviewed shell expansion in command word".to_owned());
        }
        self.pending.words.push(word.to_owned());
        Ok(())
    }

    /// Stores a redirect or rejects a second redirect that still needs a destination.
    /// For example, `>>` remains pending until the next word supplies its destination.
    fn push_redirect(&mut self, value: &str) -> Result<(), String> {
        if !matches!(value, ">>" | ">" | "<") {
            self.pending.redirects.push(value.to_owned());
            return Ok(());
        }
        if self.pending.redirect.is_some() {
            return Err("nested shell redirection is unsupported".to_owned());
        }
        self.pending.redirect = Some(value.to_owned());
        Ok(())
    }

    /// Resolves the current invocation only after validating its environment and redirects.
    fn push(&mut self, follow: Follow) -> Result<(), String> {
        let has_no_pending_command_parts = self.pending.words.is_empty()
            && self.pending.assignments.is_empty()
            && self.pending.redirects.is_empty();
        if has_no_pending_command_parts {
            if !matches!(follow, Follow::End | Follow::And) {
                return Err("unsupported empty shell command".to_owned());
            }
            return Ok(());
        }
        let pending = std::mem::take(&mut self.pending);
        let invocation = Invocation {
            words: pending.words,
            assignments: pending.assignments,
            redirects: pending.redirects,
        };
        invocation.validate_environment()?;
        validate_redirects(&invocation)?;
        let route = (self.resolver)(&invocation, self.controlled)?;
        self.commands.push(Resolved {
            route,
            follow,
            invocation,
        });
        Ok(())
    }
}

/// Keeps only the two exact quoted text arguments used by reviewed shell routes.
/// For example, the canonical token echo is accepted and other words follow normal parsing.
fn is_literal_echo_argument(words: &[String], word: &str) -> bool {
    words == ["echo"]
        && matches!(
            word,
            "cargo-nextest not installed, falling back to cargo test"
                | "available=${{ secrets.CS_ACCESS_TOKEN != '' }}"
        )
}

/// Parses all words and routes while keeping mutable state inside one bounded parser object.
pub(super) fn parse_commands<F>(
    tokens: &[Token],
    controlled: bool,
    resolver: &mut F,
) -> Result<Vec<Resolved>, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    CommandParser {
        controlled,
        resolver,
        commands: Vec::new(),
        pending: Pending::default(),
    }
    .parse(tokens)
}

/// Permits only the exact version-probe and protected-output redirects used by this consumer.
fn validate_redirects(invocation: &Invocation) -> Result<(), String> {
    let has_reviewed_redirect_route = invocation.redirects.is_empty()
        || is_nextest_probe(invocation)
        || is_token_presence(invocation);
    if has_reviewed_redirect_route {
        Ok(())
    } else {
        Err("unreviewed shell redirection".to_owned())
    }
}

/// Recognizes the exact token-presence output command without general expression exemptions.
pub(super) fn is_token_presence(invocation: &Invocation) -> bool {
    invocation.assignments.is_empty()
        && invocation.words == ["echo", "available=${{ secrets.CS_ACCESS_TOKEN != '' }}"]
        && invocation.redirects == [">>$GITHUB_OUTPUT"]
}

/// Recognizes only a literal cargo-nextest version probe and its inspected output redirects.
pub(super) fn is_nextest_probe(invocation: &Invocation) -> bool {
    invocation
        .words
        .first()
        .is_some_and(|word| word == "cargo" || word == make_tools::PROBE_CARGO)
        && invocation
            .words
            .get(1)
            .is_some_and(|word| word == "nextest")
        && invocation
            .words
            .get(2)
            .is_some_and(|word| word == "--version")
        && invocation.words.len() == 3
        && (invocation.redirects.is_empty() || invocation.redirects == [">/dev/null", "2>&1"])
}

/// Locates a keyword only when the token is a complete unquoted word.
pub(super) fn keyword(tokens: &[Token], name: &str, start: usize) -> Result<usize, String> {
    keyword_after(tokens, name, start).ok_or_else(|| format!("shell conditional has no {name}"))
}

/// Locates a word following a parser boundary without treating quoted content as syntax.
pub(super) fn keyword_after(tokens: &[Token], name: &str, start: usize) -> Option<usize> {
    tokens
        .get(start..)?
        .iter()
        .position(|token| matches!(token, Token::Word(word) if word == name))
        .map(|offset| offset + start)
}

/// Recognizes a well-formed shell assignment name before argv classification.
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty()
            && name.chars().enumerate().all(|(index, character)| {
                character == '_'
                    || character.is_ascii_alphabetic()
                    || (index > 0 && character.is_ascii_digit())
            })
    })
}
