//! Binds Make trace locations to literal source rules and their recipes.

use super::{make_output, make_source, make_syntax};

/// Binds trace locations to supported source rules and every reached recipe prefix.
pub(super) fn verify_trace(source: &str, blocks: &[make_output::TraceBlock]) -> Result<(), String> {
    let physical = source.lines().collect::<Vec<_>>();
    for block in blocks {
        verify_block(source, &physical, block)?;
    }
    Ok(())
}

/// Checks one trace block in file, location, target, prefix and non-empty-command order.
/// For example, an unreviewed source file fails before its line number is inspected.
fn verify_block(
    source: &str,
    physical: &[&str],
    block: &make_output::TraceBlock,
) -> Result<(), String> {
    if block.file != "Makefile" {
        return Err(format!(
            "GNU Make trace names an unreviewed source file: {}",
            block.file
        ));
    }
    let (rule_index, recipe_lines) = source_block_location(physical, block.line)?;
    let rule = physical.get(rule_index).copied().unwrap_or("");
    let targets = source_rule_targets(rule)?;
    verify_target(source, &targets, &block.target)?;
    verify_recipe_prefixes(physical, &recipe_lines)?;
    if block.target.is_empty() || block.command.trim().is_empty() {
        return Err(format!(
            "GNU Make trace is not bound to a recipe at line {}",
            block.line
        ));
    }
    Ok(())
}

/// Binds a literal target or lazily validates the sole recognized expanded TARGET pattern.
/// For example, a literal `test` target does not evaluate the canonical TARGET lookup.
fn verify_target(source: &str, targets: &[String], target: &str) -> Result<(), String> {
    let pattern_target = targets.iter().any(|name| name == "target/%/$(TARGET)");
    let expanded_pattern = pattern_target
        && matches!(
            target,
            "target/debug/libgauss.rlib" | "target/release/libgauss.rlib"
        )
        && make_source::has_canonical_target_definition(source)?;
    if !targets.iter().any(|name| name == target) && !expanded_pattern {
        return Err(format!(
            "GNU Make trace target is not a literal source target: {target}"
        ));
    }
    Ok(())
}

/// Checks every physical recipe prefix in encounter order, rejecting ignored errors first.
/// For example, a `-+` prefix reports ignored failure before executable dry-run syntax.
fn verify_recipe_prefixes(physical: &[&str], recipe_lines: &[usize]) -> Result<(), String> {
    for &recipe_index in recipe_lines {
        let recipe_line = physical.get(recipe_index).copied().unwrap_or("");
        let recipe = recipe_line
            .strip_prefix('\t')
            .ok_or_else(|| format!("GNU Make trace has no recipe line at {}", recipe_index + 1))?;
        let prefixes = recipe
            .chars()
            .take_while(|character| matches!(character, '@' | '-' | '+'))
            .collect::<String>();
        if prefixes.contains('-') {
            return Err(format!(
                "ignored Make recipe prefix at line {}",
                recipe_index + 1
            ));
        }
        if prefixes.contains('+') {
            return Err(format!(
                "Make + recipe can execute during dry-run at line {}",
                recipe_index + 1
            ));
        }
    }
    Ok(())
}

/// Resolves a reported rule header or any tab-prefixed recipe line in that source block.
fn source_block_location(lines: &[&str], reported: usize) -> Result<(usize, Vec<usize>), String> {
    let reported_index = reported
        .checked_sub(1)
        .ok_or_else(|| "GNU Make trace has a zero source line".to_owned())?;
    let line = lines
        .get(reported_index)
        .ok_or_else(|| format!("GNU Make trace line is outside Makefile: {reported}"))?;
    if let Some(recipes) = header_recipe_lines(lines, reported_index, line) {
        return Ok((reported_index, recipes));
    }
    for rule in (0..reported_index).rev() {
        if source_rule_targets(lines.get(rule).copied().unwrap_or("")).is_ok()
            && recipe_lines_after(lines, rule).contains(&reported_index)
        {
            return Ok((rule, recipe_lines_after(lines, rule)));
        }
    }
    Err(format!(
        "GNU Make trace location is not a literal rule or its recipe line: {reported}"
    ))
}

/// Returns recipes only when the reported source line itself is a supported rule header.
/// For example, an assignment line returns None so reverse recipe lookup can continue.
fn header_recipe_lines(lines: &[&str], index: usize, line: &str) -> Option<Vec<usize>> {
    if source_rule_targets(line).is_err() {
        return None;
    }
    let recipes = recipe_lines_after(lines, index);
    (!recipes.is_empty()).then_some(recipes)
}

/// Collects every recipe line within a physical rule block without evaluating Make syntax.
fn recipe_lines_after(lines: &[&str], rule: usize) -> Vec<usize> {
    let mut recipes = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(rule + 1) {
        let plain = line.trim();
        if line.starts_with('\t') {
            recipes.push(index);
            continue;
        }
        let is_non_recipe_content =
            plain.is_empty() || plain.starts_with('#') || is_conditional_directive(plain);
        if is_non_recipe_content {
            continue;
        }
        break;
    }
    recipes
}

/// Reads literal targets from a rule header, refusing assignments and computed names.
fn source_rule_targets(line: &str) -> Result<Vec<String>, String> {
    let plain = make_syntax::without_comment(line.trim());
    let Some(colon) = make_syntax::rule_separator(plain) else {
        return Err("trace source line is not a Make rule".to_owned());
    };
    let names = plain
        .get(..colon)
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>();
    if names.is_empty() || names.iter().any(|name| is_unbound_target(name)) {
        return Err("trace source rule has computed or pattern targets".to_owned());
    }
    Ok(names.into_iter().map(str::to_owned).collect())
}

/// Distinguishes unbound computed targets from the reviewed canonical TARGET pattern.
/// For example, `$(OTHER)` is unbound while `target/%/$(TARGET)` is inspected separately.
fn is_unbound_target(name: &str) -> bool {
    (name.contains('$') || name.contains('%')) && name != "target/%/$(TARGET)"
}

/// Identifies branch delimiters that may separate a rule header from its recipe.
fn is_conditional_directive(line: &str) -> bool {
    ["ifeq", "ifneq", "ifdef", "ifndef", "else", "endif"]
        .iter()
        .any(|prefix| line == *prefix || line.starts_with(&format!("{prefix} ")))
}

/// Builds trace evidence for a literal target without interpreting its prerequisites.
/// For example, `suite:\n\tcargo test` yields a line-1 trace for `suite`.
#[cfg(test)]
pub(super) fn trace_for_target(
    source: &str,
    target: &str,
    command: &str,
) -> Result<make_output::TraceBlock, String> {
    for (index, line) in source.lines().enumerate() {
        let plain = make_syntax::without_comment(line.trim());
        let Some(colon) = make_syntax::rule_separator(plain) else {
            continue;
        };
        let names = plain.get(..colon).unwrap_or("").split_whitespace();
        if names.into_iter().any(|name| name == target) {
            return Ok(make_output::TraceBlock {
                file: "Makefile".to_owned(),
                line: index + 1,
                target: target.to_owned(),
                command: command.to_owned(),
            });
        }
    }
    Err(format!("fixture target is missing: {target}"))
}
