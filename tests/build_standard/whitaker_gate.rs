//! Source contracts for the two-leaf lint gate.
//!
//! `make -n` prints the same commands whether or not Make may interleave them,
//! so the ordering guarantee cannot be read from a dry run. These contracts
//! read the candidate Makefile directly and refuse an edit that drops the
//! fail-fast chain, reorders or narrows the leaves, or lets the Whitaker recipe
//! swallow its own failure.

/// The candidate Makefile, read at compile time from the crate root.
const MAKEFILE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Makefile"));

/// Recipe prefixes that make a shell ignore the command's exit status.
const MASKING_FRAGMENTS: &[&str] = &[
    "|| true",
    "||true",
    "|| :",
    "||:",
    "|| exit 0",
    "; exit 0",
    "2>/dev/null",
];

/// Returns the prerequisite words of a goal, before its help comment.
fn prerequisites(goal: &str) -> Result<Vec<String>, String> {
    let prefix = format!("{goal}:");
    let rest = MAKEFILE
        .lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .ok_or_else(|| format!("Makefile no longer defines the `{goal}` goal"))?
        .split("##")
        .next()
        .unwrap_or_default();
    Ok(rest.split_whitespace().map(str::to_owned).collect())
}

/// Returns the value of a recursive Make variable assignment.
fn variable(name: &str) -> Result<&'static str, String> {
    let prefix = format!("{name} = ");
    MAKEFILE
        .lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .ok_or_else(|| format!("Makefile no longer defines the `{name}` variable"))
}

/// Returns the recipe lines indented under a goal.
fn recipe(goal: &str) -> Result<Vec<&'static str>, String> {
    let mut lines = MAKEFILE.lines();
    let header = format!("{goal}:");
    lines
        .find(|line| line.starts_with(&header))
        .ok_or_else(|| format!("Makefile no longer defines the `{goal}` goal"))?;
    Ok(lines
        .take_while(|line| line.starts_with('\t'))
        .map(|line| line.trim_start_matches('\t'))
        .collect())
}

/// The composite gate runs Clippy before Whitaker through one fail-fast chain.
///
/// A prerequisite list would be interleaved under `make -j`, and `make -k`
/// would still build the second prerequisite after the first failed, so the
/// ordering and the stop-on-failure guarantee both come from the shell chain.
#[test]
fn the_lint_goal_chains_its_two_leaves_fail_fast() -> Result<(), String> {
    let actual = prerequisites("lint")?;
    if !actual.is_empty() {
        return Err(format!(
            "`make lint` must not list prerequisites, found {actual:?}"
        ));
    }
    let chain = recipe("lint")?.join(" ");
    require_chained_leaves(&chain)?;
    if chain.contains("$(MAKE)") {
        return Err(format!("`make lint` recurses through Make: {chain}"));
    }
    require_unmasked("`make lint`", &chain)?;
    require_shared_definitions()
}

/// Requires the composite recipe to chain exactly the two shared definitions.
fn require_chained_leaves(chain: &str) -> Result<(), String> {
    let Some((first, second)) = chain.split_once("&&") else {
        return Err(format!("`make lint` does not chain its leaves: {chain}"));
    };
    if first.trim() != "$(LINT_CLIPPY_COMMANDS)" {
        return Err(format!("`make lint` does not run Clippy first: {chain}"));
    }
    if second.trim() != "$(LINT_WHITAKER_COMMANDS)" {
        return Err(format!("`make lint` does not run Whitaker second: {chain}"));
    }
    Ok(())
}

/// Requires both leaf command sequences to be present and complete.
fn require_shared_definitions() -> Result<(), String> {
    let clippy = variable("LINT_CLIPPY_COMMANDS")?;
    if !clippy.contains("$(CARGO) doc --workspace --no-deps")
        || !clippy.contains("$(CARGO) clippy $(CLIPPY_FLAGS)")
    {
        return Err(format!("`LINT_CLIPPY_COMMANDS` is incomplete: {clippy}"));
    }
    let whitaker = variable("LINT_WHITAKER_COMMANDS")?;
    if !whitaker.contains("$(WHITAKER) --all -- $(WHITAKER_PACKAGES)") {
        return Err(format!(
            "`LINT_WHITAKER_COMMANDS` is incomplete: {whitaker}"
        ));
    }
    Ok(())
}

/// Rejects a recipe or definition that could hide a command's exit status.
fn require_unmasked(label: &str, text: &str) -> Result<(), String> {
    MASKING_FRAGMENTS
        .iter()
        .find(|fragment| text.contains(**fragment))
        .map_or(Ok(()), |fragment| {
            Err(format!("{label} masks failure with {fragment:?}: {text}"))
        })
}

/// Whitaker audits the whole workspace with the packages named once in a
/// variable, so a caller can replace the scope without editing the recipe.
#[test]
fn whitaker_package_scope_is_declared_once_and_overridable() -> Result<(), String> {
    const DECLARATION: &str = "WHITAKER_PACKAGES ?= --workspace --all-targets --all-features";
    match MAKEFILE
        .lines()
        .find(|line| line.starts_with("WHITAKER_PACKAGES"))
    {
        Some(line) if line.trim_end() == DECLARATION => Ok(()),
        Some(line) => Err(format!("unexpected WHITAKER_PACKAGES declaration: {line}")),
        None => Err("Makefile does not scope the Whitaker packages".to_owned()),
    }
}

/// The Whitaker leaf and the composite chain share the same suite command, which
/// forwards the package scope and keeps its exit status.
#[test]
fn the_whitaker_recipe_runs_the_suite_and_cannot_mask_failure() -> Result<(), String> {
    let commands = variable("LINT_WHITAKER_COMMANDS")?;
    if !commands.contains("$(WHITAKER) --all -- $(WHITAKER_PACKAGES)") {
        return Err(format!(
            "`LINT_WHITAKER_COMMANDS` does not run the suite with its package scope: {commands}"
        ));
    }
    let lines = recipe("lint-whitaker")?;
    if lines != ["$(LINT_WHITAKER_COMMANDS)"] {
        return Err(format!(
            "`make lint-whitaker` does not run the shared definition: {lines:?}"
        ));
    }
    require_unmasked("the Whitaker gate", commands)?;
    lines
        .iter()
        .try_for_each(|line| require_unmasked("the Whitaker gate", line))
}

/// Both leaves are goals in their own right, so neither may be an incidental file.
#[test]
fn both_lint_leaves_are_phony_goals() -> Result<(), String> {
    let phony = MAKEFILE
        .lines()
        .skip_while(|line| !line.starts_with(".PHONY:"))
        .take_while(|line| line.starts_with(".PHONY:") || line.starts_with('\t'))
        .flat_map(|line| line.trim_start_matches(".PHONY:").split_whitespace())
        .collect::<Vec<_>>();
    for leaf in ["lint-clippy", "lint-whitaker"] {
        if !phony.contains(&leaf) {
            return Err(format!("`{leaf}` is not listed in .PHONY"));
        }
    }
    Ok(())
}
