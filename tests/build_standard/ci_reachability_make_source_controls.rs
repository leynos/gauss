//! Validates Make recipe controls and conditional selector state before any dry run.

use std::collections::BTreeSet;

use super::super::make_syntax::{rule_separator, without_comment};
use super::assignment_name;

/// Rejects global controls and recipe prefixes while preserving source encounter order.
pub(super) fn validate_controls(lines: &[String]) -> Result<(), String> {
    for line in lines {
        let plain = without_comment(line.trim());
        if plain.is_empty() || plain.starts_with('#') {
            continue;
        }
        if line.starts_with('\t') {
            validate_recipe_control(line.trim_start_matches('\t'))?;
        } else {
            validate_global_control(plain)?;
        }
    }
    Ok(())
}

/// Rejects executable dry-run prefixes; ignored errors remain a reached-trace obligation.
/// For example, `+cargo test` fails here while `-cargo test` needs trace verification.
fn validate_recipe_control(recipe: &str) -> Result<(), String> {
    if recipe.starts_with(char::is_whitespace) {
        return Err("Make recipe has ambiguous leading whitespace".to_owned());
    }
    let prefix = recipe
        .chars()
        .take_while(|character| matches!(character, '@' | '-' | '+'))
        .collect::<String>();
    if prefix.contains('+') {
        return Err("Make + recipe can execute during dry-run".to_owned());
    }
    if recursive_make(recipe) {
        return Err("recursive Make recipe can escape dry-run".to_owned());
    }
    Ok(())
}

/// Checks global directives before assignment and recursive-reference restrictions.
/// For example, an include directive is refused before any assignment is considered.
fn validate_global_control(plain: &str) -> Result<(), String> {
    let normalized = plain.trim_start_matches("export ");
    if is_include(plain) {
        return Err("uninspected Make include".to_owned());
    }
    if matches!(plain.split_whitespace().next(), Some("load" | "-load")) {
        return Err("uninspected Make load directive".to_owned());
    }
    if has_global_failure_target(plain) {
        return Err(format!("unprovable Make global directive: {plain}"));
    }
    validate_special_controls(plain, normalized)?;
    if plain.starts_with("override ") || plain.starts_with("define ") {
        return Err(format!("unprovable Make override or definition: {plain}"));
    }
    if recursive_make(plain) {
        return Err("recursive Make reference is unprovable".to_owned());
    }
    Ok(())
}

/// Refuses special target spellings before execution-environment assignments.
/// For example, `.SHELLFLAGS := -e` reports the global-directive diagnostic.
fn validate_special_controls(plain: &str, normalized: &str) -> Result<(), String> {
    if [
        ".ONESHELL",
        ".IGNORE",
        ".SILENT",
        ".SHELLFLAGS",
        ".RECIPEPREFIX",
    ]
    .iter()
    .any(|name| normalized.starts_with(&format!("{name}:")))
    {
        return Err(format!("unprovable Make global directive: {plain}"));
    }
    if matches!(
        assignment_name(normalized),
        Some(".SHELLFLAGS" | ".RECIPEPREFIX")
    ) {
        return Err(format!("unprovable Make global directive: {plain}"));
    }
    if ["SHELL", "MAKEFLAGS", "MAKEFILES", "GNUMAKEFLAGS", "MFLAGS"]
        .iter()
        .any(|name| assignment_name(normalized) == Some(*name))
    {
        return Err("Make execution environment is overridden".to_owned());
    }
    Ok(())
}

/// Finds failure-policy targets even when target lists contain surrounding whitespace.
fn has_global_failure_target(line: &str) -> bool {
    rule_separator(line)
        .and_then(|colon| line.get(..colon))
        .is_some_and(|targets| {
            targets
                .split_whitespace()
                .any(|target| matches!(target, ".ONESHELL" | ".IGNORE" | ".SILENT"))
        })
}

/// Retains the branch stack and explicitly observed selector names.
#[derive(Default)]
struct ConditionalState {
    /// One flag per open conditional, recording whether its else has been seen.
    stack: Vec<bool>,
    /// Literal selectors required from the workflow caller.
    selectors: BTreeSet<String>,
}

impl ConditionalState {
    /// Applies one directive in source order, leaving ordinary source lines unchanged.
    /// For example, an unmatched `else` fails without adding a branch frame.
    fn observe(&mut self, plain: &str) -> Result<(), String> {
        if plain.starts_with("ifeq") || plain.starts_with("ifneq") {
            self.open(plain)?;
        } else if unsupported_conditional(plain) {
            return Err(format!("unsupported Make conditional form: {plain}"));
        } else if plain == "else" {
            self.accept_else()?;
        } else if plain.starts_with("else") {
            return Err(format!("unsupported Make conditional form: {plain}"));
        } else if plain == "endif" {
            self.close()?;
        } else if plain.starts_with("endif") {
            return Err(format!("unsupported Make conditional form: {plain}"));
        }
        Ok(())
    }

    /// Validates the selector before recording a new branch frame.
    /// For example, a literal MODE equality records MODE and opens one frame.
    fn open(&mut self, plain: &str) -> Result<(), String> {
        let (name, expected) = condition(plain)?;
        if !matches!(name, "MODE" | "BUILD_HOST_OS") {
            return Err(format!("tool-dependent or unknown Make selector: {name}"));
        }
        if expected.is_empty() {
            return Err("Make conditional has an empty comparison value".to_owned());
        }
        self.selectors.insert(name.to_owned());
        self.stack.push(false);
        Ok(())
    }

    /// Marks the innermost else, refusing an unmatched or repeated delimiter.
    /// For example, the second `else` in one frame reports a repeated branch.
    fn accept_else(&mut self) -> Result<(), String> {
        let seen = self
            .stack
            .last_mut()
            .ok_or_else(|| "Make else has no matching conditional".to_owned())?;
        if *seen {
            return Err("Make conditional has multiple else branches".to_owned());
        }
        *seen = true;
        Ok(())
    }

    /// Closes exactly one branch frame.
    /// For example, `endif` with no open frame reports an unmatched conditional.
    fn close(&mut self) -> Result<(), String> {
        if self.stack.pop().is_none() {
            return Err("Make endif has no matching conditional".to_owned());
        }
        Ok(())
    }

    /// Returns selectors only after every branch frame has closed.
    /// For example, an open MODE frame reports the missing-endif diagnostic.
    fn finish(self) -> Result<BTreeSet<String>, String> {
        if self.stack.is_empty() {
            Ok(self.selectors)
        } else {
            Err("Make conditional is missing endif".to_owned())
        }
    }
}

/// Validates conditional shape and returns only explicitly supported selector names.
pub(super) fn validate_conditionals(lines: &[String]) -> Result<BTreeSet<String>, String> {
    let mut state = ConditionalState::default();
    for line in lines {
        state.observe(without_comment(line.trim()))?;
    }
    state.finish()
}

/// Identifies unsupported opening and chained conditional forms before delimiter handling.
/// For example, `else ifeq` is refused rather than treated as a second branch.
fn unsupported_conditional(plain: &str) -> bool {
    ["ifdef ", "ifndef ", "else if", "else ifeq", "else ifneq"]
        .iter()
        .any(|prefix| plain.starts_with(prefix))
}

/// Parses only parenthesized equality checks against a literal scalar.
fn condition(line: &str) -> Result<(&str, &str), String> {
    let body = line
        .strip_prefix("ifeq")
        .or_else(|| line.strip_prefix("ifneq"))
        .and_then(|tail| tail.trim().strip_prefix('('))
        .and_then(|tail| tail.strip_suffix(')'))
        .ok_or_else(|| format!("unsupported Make condition: {line}"))?;
    let (selector, value) = body
        .split_once(',')
        .ok_or_else(|| format!("unresolved Make condition: {line}"))?;
    let name = selector
        .trim()
        .strip_prefix("$(")
        .and_then(|text| text.strip_suffix(')'))
        .ok_or_else(|| format!("unresolved Make selector: {selector}"))?;
    let comparison = value.trim();
    if comparison
        .chars()
        .any(|character| matches!(character, '$' | '(' | ')' | ','))
    {
        return Err(format!("computed Make condition value: {line}"));
    }
    Ok((name, comparison))
}

/// Recognizes all Make include directive spellings.
fn is_include(line: &str) -> bool {
    line.split_whitespace()
        .next()
        .is_some_and(|word| matches!(word, "include" | "-include" | "sinclude"))
}

/// Detects recursive Make references in either supported delimiter spelling.
fn recursive_make(text: &str) -> bool {
    text.contains("$(MAKE)") || text.contains("${MAKE}")
}
