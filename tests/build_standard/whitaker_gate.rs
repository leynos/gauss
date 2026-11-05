//! Source contracts for the two-leaf lint gate.
//!
//! `make -n` prints the same commands whether or not Make may interleave them,
//! so the ordering guarantee cannot be read from a dry run. These contracts
//! read the candidate Makefile directly and refuse an edit that drops the
//! serialization directive, reorders or narrows the prerequisite list, or lets
//! the Whitaker recipe swallow its own failure.

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

/// The composite gate runs Clippy before Whitaker by listing them in that order.
#[test]
fn the_lint_goal_orders_its_two_leaves() -> Result<(), String> {
    let actual = prerequisites("lint")?;
    let expected = ["lint-clippy", "lint-whitaker"];
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "`make lint` prerequisites are {actual:?}, not {expected:?}"
        ))
    }
}

/// Make would interleave an unordered prerequisite list under `-j`, so the goal
/// must opt out of parallel prerequisite execution.
#[test]
fn the_lint_goal_holds_its_leaves_in_order_under_parallel_make() -> Result<(), String> {
    let count = MAKEFILE
        .lines()
        .filter(|line| line.trim_end() == ".NOTPARALLEL: lint")
        .count();
    if count == 1 {
        Ok(())
    } else {
        Err(format!(
            "expected one `.NOTPARALLEL: lint` directive, found {count}"
        ))
    }
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

/// The Whitaker recipe must forward the package scope and keep its exit status.
#[test]
fn the_whitaker_recipe_runs_the_suite_and_cannot_mask_failure() -> Result<(), String> {
    let lines = recipe("lint-whitaker")?;
    let invocation = lines
        .iter()
        .find(|line| line.contains("$(WHITAKER) --all -- $(WHITAKER_PACKAGES)"));
    let Some(suite_line) = invocation else {
        return Err(format!(
            "`make lint-whitaker` does not run the suite with its package scope: {lines:?}"
        ));
    };
    if suite_line.starts_with('-') {
        return Err("`make lint-whitaker` ignores the suite's exit status".to_owned());
    }
    for line in lines {
        if let Some(fragment) = MASKING_FRAGMENTS
            .iter()
            .find(|fragment| line.contains(**fragment))
        {
            return Err(format!(
                "`make lint-whitaker` masks failure with {fragment:?}: {line}"
            ));
        }
    }
    Ok(())
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
