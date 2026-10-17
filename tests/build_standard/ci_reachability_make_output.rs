//! Binds GNU Make trace blocks to shell commands and resolves every invocation.

use super::{
    Invocation, Route,
    make_syntax::{self, Token},
    make_tools,
};
use std::ops::Range;

use command_parser::{
    Follow, Resolved, is_nextest_probe, is_token_presence, keyword, keyword_after, parse_commands,
};

/// Parses command rows and shell status delimiters for the parent classifier.
#[path = "ci_reachability_make_output_parser.rs"]
mod command_parser;

/// A trace-bound recipe command emitted by GNU Make's future `--trace` adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TraceBlock {
    /// The Makefile identity as reported by the trace.
    pub(super) file: String,
    /// The one-based source line reported by GNU Make's trace.
    pub(super) line: usize,
    /// The target whose recipe GNU Make selected.
    pub(super) target: String,
    /// The complete printed recipe body, including explicit continuations.
    pub(super) command: String,
}

/// Resolved route plus every traced recipe and executable invocation.
#[derive(Debug)]
pub(super) struct Evaluation {
    /// Combined suite reachability after all command rows were classified.
    pub(super) route: Route,
    /// Trace rows in the order printed by GNU Make.
    pub(super) blocks: Vec<TraceBlock>,
    /// Every invocation encountered, including both conditional branches.
    pub(super) invocations: Vec<Invocation>,
}

/// Classifies an untraced command stream without accepting private Make probes.
pub(super) fn classify(output: &str) -> Result<Route, String> {
    classify_with(output, false, |invocation, controlled| {
        resolve(invocation, controlled, None)
    })
}

/// Resolves each untraced shell command while retaining command and conditional boundaries.
pub(super) fn classify_with<F>(
    output: &str,
    controlled: bool,
    mut resolver: F,
) -> Result<Route, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    let lines = make_syntax::shell_lines(output)?;
    if lines.is_empty() {
        return Err("shell command stream is empty".to_owned());
    }
    let mut route = Route::NonSuite;
    for line in lines {
        route = route.combine(classify_line(&line, controlled, &mut resolver)?);
    }
    Ok(route)
}

/// Parses the full Make trace, binds source lines, then inspects every printed command row.
pub(super) fn classify_traced<F>(
    output: &str,
    source: &str,
    controlled: bool,
    mut resolver: F,
) -> Result<Evaluation, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    let blocks = trace_blocks(output)?;
    super::make_routes::verify_trace(source, &blocks)?;
    let mut route = Route::NonSuite;
    let mut invocations = Vec::new();
    for block in &blocks {
        let block_route = classify_with(&block.command, controlled, |invocation, selected| {
            invocations.push(invocation.clone());
            resolver(invocation, selected)
        })?;
        route = route.combine(block_route);
    }
    Ok(Evaluation {
        route,
        blocks,
        invocations,
    })
}

/// Collects every command line beneath a strict trace header; unknown output is a reader gap.
fn trace_blocks(output: &str) -> Result<Vec<TraceBlock>, String> {
    let mut blocks = Vec::new();
    let mut active: Option<TraceBlock> = None;
    for line in output.lines() {
        if let Some(header) = trace_header(line)? {
            if let Some(block) = active.take() {
                blocks.push(block);
            }
            active = Some(header);
        } else {
            append_trace_line(line, &mut active)?;
        }
    }
    if let Some(block) = active {
        blocks.push(block);
    }
    if blocks.is_empty() {
        Err("GNU Make emitted no recognized trace blocks".to_owned())
    } else {
        Ok(blocks)
    }
}

/// Appends one non-header trace line or rejects output without an active target.
/// For example, blank lines are ignored and recipe text extends the current trace block.
fn append_trace_line(line: &str, active: &mut Option<TraceBlock>) -> Result<(), String> {
    if line.trim().is_empty() {
        return Ok(());
    }
    let Some(block) = active.as_mut() else {
        return Err(format!("unbound GNU Make trace output: {line}"));
    };
    if !block.command.is_empty() {
        block.command.push('\n');
    }
    block.command.push_str(line);
    Ok(())
}

/// Distinguishes GNU Make's prerequisite trace from its empty-prerequisite trace.
/// Both forms still require a literal source target and recipe binding.
enum TraceHeaderTail<'a> {
    /// Target and changed prerequisites printed after `update target`.
    Prerequisites(&'a str),
    /// Target with the exact missing-target suffix and no prerequisite text.
    MissingTarget(&'a str),
}

impl<'a> TraceHeaderTail<'a> {
    /// Recognizes the two reviewed alternatives, with prerequisite syntax taking priority.
    /// For example, an unsupported header returns the existing unrecognized-header gap.
    fn recognize(rest: &'a str) -> Result<(&'a str, Self), String> {
        rest.split_once(": update target '")
            .map(|(number, tail)| (number, Self::Prerequisites(tail)))
            .or_else(|| {
                rest.split_once(": target '")
                    .map(|(number, tail)| (number, Self::MissingTarget(tail)))
            })
            .ok_or_else(|| "unrecognized GNU Make trace header".to_owned())
    }

    /// Extracts the target and optional prerequisites without inventing a reason.
    /// For example, `MissingTarget("test' does not exist")` yields `("test", None)`.
    fn parts(self) -> Result<(&'a str, Option<&'a str>), String> {
        let parts = match self {
            Self::Prerequisites(tail) => tail
                .split_once("' due to: ")
                .map(|(target, reason)| (target, Some(reason))),
            Self::MissingTarget(tail) => tail
                .strip_suffix("' does not exist")
                .map(|target| (target, None)),
        };
        parts.ok_or_else(|| "GNU Make trace target or prerequisite is unbound".to_owned())
    }

    /// Validates printed location fields before constructing a block for later source binding.
    /// For example, a nonnumeric line fails before malformed target or reason text is checked.
    fn into_block(self, number: &str) -> Result<TraceBlock, String> {
        let line_number = number
            .parse::<usize>()
            .map_err(|_| "GNU Make trace line is not numeric".to_owned())?;
        let (target, reason) = self.parts()?;
        let is_incomplete_trace_location = line_number == 0
            || target.is_empty()
            || reason.is_some_and(|text| text.trim().is_empty());
        if is_incomplete_trace_location {
            return Err("GNU Make trace has an empty location, target, or reason".to_owned());
        }
        Ok(TraceBlock {
            file: "Makefile".to_owned(),
            line: line_number,
            target: target.to_owned(),
            command: String::new(),
        })
    }
}

/// Parses the two exact GNU Make 4.3 target trace forms before source binding.
fn trace_header(line: &str) -> Result<Option<TraceBlock>, String> {
    let Some(rest) = line.strip_prefix("Makefile:") else {
        let names_foreign_source = line.contains(": update target ") || line.contains(": target '");
        return if names_foreign_source {
            Err("GNU Make trace names an unreviewed source file".to_owned())
        } else {
            Ok(None)
        };
    };
    let (number, header_tail) = TraceHeaderTail::recognize(rest)?;
    header_tail.into_block(number).map(Some)
}

/// Classifies a complete shell line, checking both availability branches of the current wrapper.
fn classify_line<F>(line: &str, controlled: bool, resolver: &mut F) -> Result<Route, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    let tokens = make_syntax::tokenize(line)?;
    if matches!(tokens.first(), Some(Token::Word(word)) if word == "if") {
        classify_conditional(&tokens, controlled, resolver)
    } else {
        classify_commands(&tokens, controlled, resolver)
    }
}

/// Requires the exact Nextest probe and a failure-preserving suite in each branch.
fn classify_conditional<F>(
    tokens: &[Token],
    controlled: bool,
    resolver: &mut F,
) -> Result<Route, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    let then_at = keyword(tokens, "then", 1)?;
    let else_at = keyword(tokens, "else", then_at + 1)?;
    let fi_at = keyword(tokens, "fi", else_at + 1)?;
    if fi_at + 1 != tokens.len() || keyword_after(tokens, "if", 1).is_some() {
        return Err("nested or trailing shell conditional is unsupported".to_owned());
    }
    let condition = commands_in(tokens, 1..then_at, controlled, resolver)?;
    let (probe, rest) = condition
        .split_first()
        .ok_or_else(|| "Nextest condition has no probe command".to_owned())?;
    if !rest.is_empty() || !is_nextest_probe(&probe.invocation) {
        return Err("shell conditional is not the exact Nextest availability probe".to_owned());
    }
    let then_commands = commands_in(tokens, then_at + 1..else_at, controlled, resolver)?;
    let else_commands = commands_in(tokens, else_at + 1..fi_at, controlled, resolver)?;
    let then_route = require_final_suite(&then_commands, "Nextest branch")?;
    let else_route = require_fallback_suite(&else_commands)?;
    Ok(probe.route.combine(then_route).combine(else_route))
}

/// Extracts commands between a pair of already located shell keywords.
fn commands_in<F>(
    tokens: &[Token],
    span: Range<usize>,
    controlled: bool,
    resolver: &mut F,
) -> Result<Vec<Resolved>, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    let mut branch = tokens
        .get(span)
        .ok_or_else(|| "invalid shell branch boundary".to_owned())?
        .to_vec();
    if matches!(branch.last(), Some(Token::Semi)) {
        let _ = branch.pop();
    }
    parse_commands(&branch, controlled, resolver)
}

/// Requires the final command to be a suite, preserving its result through the shell branch.
fn require_final_suite(commands: &[Resolved], context: &str) -> Result<Route, String> {
    let (last, preceding) = commands
        .split_last()
        .ok_or_else(|| format!("{context} has no executable command"))?;
    if last.route != Route::Suite || preceding.iter().any(|item| item.route == Route::Suite) {
        return Err(format!("{context} does not preserve the suite result"));
    }
    if preceding.is_empty()
        || preceding.iter().all(|item| {
            item.route == Route::NonSuite
                && item
                    .invocation
                    .words
                    .first()
                    .is_some_and(|word| word == "echo")
                && item.follow == Follow::Semi
        })
    {
        Ok(Route::Suite)
    } else {
        Err(format!("{context} contains a failure-masking command"))
    }
}

/// Requires the canonical fallback notice followed by a Cargo test invocation.
fn require_fallback_suite(commands: &[Resolved]) -> Result<Route, String> {
    let route = require_final_suite(commands, "Cargo fallback branch")?;
    let valid = commands.len() == 2
        && commands.first().is_some_and(|notice| {
            notice.invocation.words
                == [
                    "echo",
                    "cargo-nextest not installed, falling back to cargo test",
                ]
                && notice.follow == Follow::Semi
        })
        && commands.last().is_some_and(|suite| {
            suite
                .invocation
                .words
                .get(1)
                .is_some_and(|word| word == "test")
        });
    if valid {
        Ok(route)
    } else {
        Err("Cargo fallback branch is not the inspected echo/test form".to_owned())
    }
}

/// Resolves all commands before rejecting delimiters that can mask a suite result.
fn classify_commands<F>(
    tokens: &[Token],
    controlled: bool,
    resolver: &mut F,
) -> Result<Route, String>
where
    F: FnMut(&Invocation, bool) -> Result<Route, String>,
{
    let commands = parse_commands(tokens, controlled, resolver)?;
    if commands
        .iter()
        .any(|command| !matches!(command.follow, Follow::End | Follow::And))
    {
        return Err("shell sequence can hide a command failure".to_owned());
    }
    Ok(commands.iter().fold(Route::NonSuite, |route, command| {
        route.combine(command.route)
    }))
}

/// Resolves exact token presence, then delegates every other command to the narrow tool registry.
pub(super) fn resolve(
    invocation: &Invocation,
    controlled: bool,
    workspace: Option<&super::Workspace<'_>>,
) -> Result<Route, String> {
    if is_token_presence(invocation) {
        return Ok(Route::NonSuite);
    }
    if invocation.words
        == [
            "echo",
            "cargo-nextest not installed, falling back to cargo test",
        ]
        && invocation.redirects.is_empty()
    {
        return Ok(Route::NonSuite);
    }
    if !invocation.redirects.is_empty() && !is_nextest_probe(invocation) {
        return Err("unreviewed shell redirection".to_owned());
    }
    workspace.map_or_else(
        || make_tools::classify_words(&invocation.words, controlled),
        |root| make_tools::classify_workspace_words(&invocation.words, controlled, root),
    )
}
