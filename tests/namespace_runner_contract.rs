//! Guard Gauss's reviewed Linux runner assignments.

const NAMESPACE_RUNNER: &str = "namespace-profile-default";

/// Return the YAML block for one named workflow job.
fn job_block<'workflow>(workflow: &'workflow str, job_name: &str) -> Option<&'workflow str> {
    let marker = format!("\n  {job_name}:\n");
    let body = workflow.split_once(&marker)?.1;
    Some(body.split_once("\n  ").map_or(body, |(job, _)| job))
}

#[test]
fn main_coverage_uses_the_shared_namespace_profile() {
    let workflow = include_str!("../.github/workflows/coverage-main.yml");
    let coverage_job = job_block(workflow, "coverage-upload");
    assert!(
        coverage_job.is_some_and(|job| job
            .lines()
            .any(|line| { line.trim() == format!("runs-on: {NAMESPACE_RUNNER}") })),
        "coverage-upload must use {NAMESPACE_RUNNER}"
    );
}

#[test]
fn ci_uses_a_github_hosted_runner_for_the_whitaker_toolchain() {
    let workflow = include_str!("../.github/workflows/ci.yml");
    let build_test_job = job_block(workflow, "build-test");
    assert!(
        build_test_job.is_some_and(|job| job
            .lines()
            .any(|line| { line.trim() == "runs-on: ubuntu-latest" })),
        "build-test requires ubuntu-latest for Whitaker's prebuilt cargo-dylint"
    );
}
