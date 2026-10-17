//! Refuses unsafe Make source before delegating dependency and branch resolution to GNU Make.

use std::collections::{BTreeMap, BTreeSet};

/// Owns recipe-control and conditional-state validation.
#[path = "ci_reachability_make_source_controls.rs"]
mod controls;

use super::make_syntax::{
    rule_separator, trailing_backslash, valid_variable_name, without_comment,
};

/// Rejects parse-time execution, Makefile remaking and global failure-policy changes.
pub(super) fn validate(source: &str) -> Result<(), String> {
    let lines = make_lines(source)?;
    validate_execution_functions(&lines)?;
    controls::validate_controls(&lines)?;
    validate_makefile_rules(source)?;
    controls::validate_conditionals(&lines).map(|_| ())
}

/// Returns supported literal selectors without choosing a branch locally.
pub(super) fn required_selectors(source: &str) -> Result<BTreeSet<String>, String> {
    controls::validate_conditionals(&make_lines(source)?)
}

/// Joins physical Make continuations while retaining recipe indentation.
fn make_lines(source: &str) -> Result<Vec<String>, String> {
    let mut output = Vec::new();
    let mut pending = String::new();
    for physical in source.lines() {
        let mut fragment = if pending.is_empty() {
            physical.to_owned()
        } else {
            physical.trim_start().to_owned()
        };
        if trailing_backslash(&fragment) {
            fragment.pop();
            pending.push_str(fragment.trim_end());
            pending.push(' ');
        } else {
            pending.push_str(&fragment);
            output.push(std::mem::take(&mut pending));
        }
    }
    if pending.is_empty() {
        Ok(output)
    } else {
        Err("Make source ends inside a continuation".to_owned())
    }
}

/// Rejects every execution-capable Make function except the two inspected lookups.
fn validate_execution_functions(lines: &[String]) -> Result<(), String> {
    let allowed = [
        "BUILD_HOST_OS := $(shell uname -s)",
        r#"MDLINT ?= $(shell command -v markdownlint-cli2 2>/dev/null || printf '%s' "$$HOME/.bun/bin/markdownlint-cli2")"#,
    ];
    for line in lines {
        let plain = without_comment(line).trim();
        if allowed.contains(&plain) {
            continue;
        }
        let checked = plain;
        if let Some(function) = unreviewed_execution_function(checked) {
            return Err(format!("unreviewed Make execution function: {function}"));
        }
        if checked.contains("!=") {
            return Err("Make shell assignment is unreviewed".to_owned());
        }
    }
    Ok(())
}

/// Finds the first execution-capable function in the fixed inspection priority.
/// For example, a line containing both `$(eval` and `$(shell` reports `shell` first.
fn unreviewed_execution_function(checked: &str) -> Option<&'static str> {
    ["shell", "eval", "file", "load", "guile", "call"]
        .into_iter()
        .find(|function| {
            checked.contains(&format!("$({function}"))
                || checked.contains(&format!("${{{function}"))
        })
}

/// Refuses explicit or pattern rules that could remake the selected Makefile under `-B`.
fn validate_makefile_rules(source: &str) -> Result<(), String> {
    let target_value = has_canonical_target_definition(source)?;
    for line in make_lines(source)? {
        validate_rule_line(&line, target_value)?;
    }
    Ok(())
}

/// Refuses inline recipes before allowing ordinary assignments or checking rule targets.
/// For example, `check:; cargo test` fails before target inspection.
fn validate_rule_line(line: &str, target_value: bool) -> Result<(), String> {
    let plain = without_comment(line).trim();
    if plain.is_empty() || line.starts_with('\t') {
        return Ok(());
    }
    if let Some(colon) = rule_separator(plain)
        && plain
            .get(colon.saturating_add(1)..)
            .is_some_and(|tail| tail.contains(';'))
    {
        return Err("inline Make recipe cannot be proved safe under dry-run".to_owned());
    }
    if assignment_name(plain).is_some() {
        return Ok(());
    }
    let Some(colon) = rule_separator(plain) else {
        return Ok(());
    };
    for target in plain.get(..colon).unwrap_or("").split_whitespace() {
        validate_rule_target(target, target_value)?;
    }
    Ok(())
}

/// Checks one target against remaking hazards, retaining the sole canonical pattern exception.
/// For example, `.DEFAULT` fails while an inspected canonical TARGET pattern passes.
fn validate_rule_target(target: &str, target_value: bool) -> Result<(), String> {
    if target == ".DEFAULT" {
        return Err("Make .DEFAULT can remake the selected Makefile".to_owned());
    }
    if target.starts_with('.') && target.matches('.').count() >= 2 {
        return Err(format!(
            "source-defined Make suffix rule is unprovable: {target}"
        ));
    }
    if target == "target/%/$(TARGET)" && target_value {
        return Ok(());
    }
    if target.contains('$') {
        return Err(format!(
            "computed Make target cannot be proved safe: {target}"
        ));
    }
    if could_match_makefile(target) {
        return Err(format!("Make rule could remake its Makefile: {target}"));
    }
    Ok(())
}

/// Checks whether a literal single-percent target could name `Makefile`.
fn could_match_makefile(target: &str) -> bool {
    if target == "Makefile" {
        return true;
    }
    let Some((prefix, suffix)) = target.split_once('%') else {
        return false;
    };
    "Makefile"
        .strip_prefix(prefix)
        .is_some_and(|tail| tail.ends_with(suffix))
        && "Makefile".len() >= prefix.len().saturating_add(suffix.len())
}

/// Identifies an assignment name before its value is interpreted by Make.
fn assignment_name(line: &str) -> Option<&str> {
    assignment_parts(line).map(|(name, _, _)| name)
}

/// Parses a simple variable assignment without mistaking rule recipes for assignments.
fn assignment_parts(line: &str) -> Option<(&str, &str, &str)> {
    let trimmed = line.trim();
    let text = trimmed.strip_prefix("export ").unwrap_or(trimmed);
    let equals = text.find('=')?;
    let through_equals = text.get(..equals.saturating_add(1))?;
    let after = text.get(equals.saturating_add(1)..)?;
    let (raw_name, operator) = ["?=", ":=", "+=", "!="]
        .into_iter()
        .find_map(|operator| {
            through_equals
                .strip_suffix(operator)
                .map(|name| (name, operator))
        })
        .unwrap_or((text.get(..equals)?, "="));
    let variable_name = raw_name.trim();
    if !valid_variable_name(variable_name) {
        return None;
    }
    Some((variable_name, operator, after.trim()))
}

/// Counts canonical target definitions while tracking their source conditional depth.
#[derive(Default)]
struct TargetDefinitions {
    /// Number of currently open equality conditionals.
    conditional_depth: usize,
    /// Number of accepted unconditional canonical definitions.
    definitions: usize,
}

impl TargetDefinitions {
    /// Observes one source line, checking TARGET definitions at their original position.
    /// For example, `ifeq` opens a frame before a following TARGET definition is checked.
    fn observe(&mut self, plain: &str) -> Result<(), String> {
        if plain.starts_with("ifeq") || plain.starts_with("ifneq") {
            self.conditional_depth += 1;
        } else if plain == "endif" {
            self.conditional_depth = self.conditional_depth.saturating_sub(1);
        } else if assignment_name(plain) == Some("TARGET") {
            self.accept_definition(plain)?;
        }
        Ok(())
    }

    /// Records a canonical definition only after its unconditional placement is established.
    /// For example, `TARGET ?= libgauss.rlib` is accepted only at depth zero.
    fn accept_definition(&mut self, plain: &str) -> Result<(), String> {
        if self.conditional_depth != 0 {
            return Err("conditional Make TARGET definition is unprovable".to_owned());
        }
        if plain != "TARGET ?= libgauss.rlib" {
            return Err("Make TARGET definition is not canonical".to_owned());
        }
        self.definitions += 1;
        Ok(())
    }
}

/// Requires one unconditional canonical TARGET before its pattern exception is used.
pub(super) fn has_canonical_target_definition(source: &str) -> Result<bool, String> {
    let mut state = TargetDefinitions::default();
    for line in make_lines(source)? {
        state.observe(without_comment(line.trim()))?;
    }
    if state.definitions != 1 {
        return Err("Make TARGET definition is missing or not unique".to_owned());
    }
    Ok(true)
}

/// Proves the executable defaults before command-line values replace them with probes.
pub(super) fn verify_tool_defaults(source: &str) -> Result<(), String> {
    let expected = BTreeMap::from([
        ("CARGO", "$(or $(wildcard $(HOME)/.cargo/bin/cargo),cargo)"),
        (
            "MDLINT",
            r#"$(shell command -v markdownlint-cli2 2>/dev/null || printf '%s' "$$HOME/.bun/bin/markdownlint-cli2")"#,
        ),
        ("MDTABLEFIX", "mdtablefix"),
        ("NIXIE", "nixie"),
        ("UV", "uv"),
        (
            "WHITAKER",
            "$(or $(wildcard $(HOME)/.local/bin/whitaker),whitaker)",
        ),
    ]);
    let definitions = collect_tool_defaults(source, &expected)?;
    for name in expected.keys() {
        match definitions.get(name) {
            Some(entries) if entries.len() == 1 => {}
            Some(_) => return Err(format!("duplicate Make tool default: {name}")),
            None => return Err(format!("missing Make tool default: {name}")),
        }
    }
    Ok(())
}

/// Collects tool assignments in source order before sorted completeness validation.
/// For example, a conditional CARGO assignment fails before missing-tool checks.
fn collect_tool_defaults<'a>(
    source: &'a str,
    expected: &BTreeMap<&str, &str>,
) -> Result<ToolDefinitions<'a>, String> {
    let mut definitions = ToolDefinitions::new();
    let mut conditional_depth = 0_usize;
    for line in source.lines() {
        let text = line.trim();
        if text.starts_with("ifeq") || text.starts_with("ifneq") {
            conditional_depth += 1;
            continue;
        }
        if text == "endif" {
            conditional_depth = conditional_depth.saturating_sub(1);
            continue;
        }
        if is_non_assignment_line(line, text) {
            continue;
        }
        record_tool_default(text, conditional_depth, expected, &mut definitions)?;
    }
    Ok(definitions)
}

/// Retains all definitions of each observed tool so duplicate assignments remain detectable.
type ToolDefinitions<'a> = BTreeMap<&'a str, Vec<(&'a str, &'a str)>>;

/// Excludes recipes, comments and empty lines from source tool assignment collection.
/// For example, a tab-prefixed `CARGO ?=` is recipe text, not a tool default.
fn is_non_assignment_line(line: &str, text: &str) -> bool {
    line.starts_with('\t') || text.is_empty() || text.starts_with('#')
}

/// Records one canonical tool assignment or reports an unsafe conditional default.
/// For example, an unconditional `CARGO ?=` is recorded, while a conditional assignment fails.
fn record_tool_default<'a>(
    text: &'a str,
    conditional_depth: usize,
    expected: &BTreeMap<&str, &str>,
    definitions: &mut ToolDefinitions<'a>,
) -> Result<(), String> {
    let assignment_text = text.strip_prefix("export ").unwrap_or(text);
    let Some((name, operator, value)) = source_assignment(assignment_text) else {
        return Ok(());
    };
    let variable = name.rsplit(':').next().map_or(name, str::trim);
    let Some(canonical) = expected.get(variable) else {
        return Ok(());
    };
    if conditional_depth > 0 || name != variable {
        return Err(format!(
            "conditional or target-specific tool default: {variable}"
        ));
    }
    definitions
        .entry(variable)
        .or_default()
        .push((operator, value));
    if value != *canonical || operator != "?=" {
        return Err(format!("unreviewed Make tool default: {variable}"));
    }
    Ok(())
}

/// Reads a plain Make assignment without interpreting its value or conditional context.
fn source_assignment(text: &str) -> Option<(&str, &str, &str)> {
    ["?=", ":=", "+=", "!=", "="]
        .into_iter()
        .find_map(|operator| {
            let (raw_name, value) = text.split_once(operator)?;
            let name = raw_name.trim();
            (!name.is_empty()).then_some((name, operator, value.trim()))
        })
}

/// Exercises the Make source guard with focused, source-backed mutations.
#[cfg(test)]
#[path = "ci_reachability_make_source_tests.rs"]
mod tests;
