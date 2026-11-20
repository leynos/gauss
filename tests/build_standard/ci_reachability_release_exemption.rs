//! Bounds the release-workflow exemption for the suite-provisioning contract.
//!
//! The release and packaging workflows build and package artefacts rather than
//! test them, and they call packaging actions the build standard never
//! reviewed. They are therefore outside the suite contract by exact path. This
//! module owns that boundary and, crucially, still proves the exempt workflows
//! cannot reach a test suite, so the exemption cannot quietly become a hole.

use super::{super::Value, Workflow, context::jobs, require_execution_context, runner};

/// The release and packaging workflows, which build and package artefacts but
/// never run a test suite.
///
/// These are exact paths rather than a pattern, so a workflow added later is
/// still inspected in full and a renamed file fails closed instead of silently
/// inheriting the exemption. The release path is deliberately outside the build
/// standard: a release stays on the platform linker by design
/// ([ADR 0006](../../docs/adrs/adr-0006-rust-build-standard.md)) and assigns
/// `RUSTFLAGS` itself, so its steps call packaging actions the build standard
/// never reviewed. Exemption is not a blanket skip: [`exempt_release_workflow`]
/// still proves every job declares a literal runner and that no step reaches a
/// suite, so a release workflow that starts testing fails this contract.
pub(super) const RELEASE_WORKFLOWS: &[&str] = &[
    ".github/workflows/build-and-package.yml",
    ".github/workflows/release.yml",
    ".github/workflows/release-dry-run.yml",
];

/// Returns whether an inventory path is an exempt release workflow.
///
/// For example, `.github/workflows/release.yml` is exempt, while
/// `.github/workflows/release-copy.yml` is not and is inspected in full.
pub(super) fn is_release_workflow(path: &str) -> bool { RELEASE_WORKFLOWS.contains(&path) }

/// Proves an exempt release workflow cannot reach a test suite.
///
/// The exemption widens which *actions* may appear, not whether a suite may run.
/// Every job therefore still needs a literal, provable runner, and no step may
/// name the suite routes this contract exists to guard. Checking for those
/// markers here means a release workflow that adds `cargo test`,
/// `cargo nextest run`, or `generate-coverage` fails instead of being skipped.
pub(super) fn exempt_release_workflow(workflow: &Workflow) -> Result<(), String> {
    for (job_id, job) in jobs(workflow)? {
        let context = format!("{}:{job_id}", workflow.path);
        require_execution_context(job).map_err(|error| format!("{context}: {error}"))?;
        if job.field("uses").is_some() {
            // A reusable call is checked through its edge in `event_reachable_workflows`.
            continue;
        }
        runner::hosts(job).map_err(|error| format!("{context}: {error}"))?;
        reject_suite_step(job, &context)?;
    }
    Ok(())
}

/// Fails when any step in the job names a suite route.
///
/// The exemption widens which actions may appear in a release workflow, so the
/// suite check stays per step rather than per job. It is a separate function
/// because the job walk's other work is unrelated, and keeping the two
/// together reads as one long ladder.
fn reject_suite_step(job: &Value, context: &str) -> Result<(), String> {
    for step in job.required("steps", "job")?.sequence("steps")? {
        let names_suite = step
            .optional_text("uses", "step uses")?
            .is_some_and(names_suite_action)
            || step
                .optional_text("run", "step run")?
                .is_some_and(names_suite_command);
        if names_suite {
            return Err(format!(
                "{context}: exempt release workflow must not run a test suite"
            ));
        }
    }
    Ok(())
}

/// Recognizes the coverage action that runs the suite for this repository.
fn names_suite_action(reference: &str) -> bool {
    reference.starts_with("leynos/shared-actions/.github/actions/generate-coverage@")
}

/// Recognizes an executable Cargo or Nextest suite invocation in a shell step.
///
/// For example, `cargo nextest run --workspace` and `cargo test --doc` both
/// name a suite, while `cargo build --release` and `cargo fmt --check` do not.
/// The check is deliberately narrow: it catches a suite being introduced into
/// a release workflow, and does not try to resolve every shell spelling. A
/// suite hidden behind a variable or a script is still caught by that script's
/// own review, and `generate-coverage` is matched exactly by
/// [`names_suite_action`].
fn names_suite_command(run: &str) -> bool {
    let words = run.split_whitespace().collect::<Vec<_>>();
    words
        .windows(2)
        .any(|pair| matches!(pair, ["cargo", "test"] | ["nextest", "run"]))
}

/// Keeps the exemption's cases beside the module they exercise.
#[cfg(test)]
#[path = "ci_reachability_release_exemption_tests.rs"]
mod tests;
