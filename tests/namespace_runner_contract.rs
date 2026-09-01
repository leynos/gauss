//! Guard Gauss's reviewed Linux runner assignments.

const NAMESPACE_RUNNER: &str = "runs-on: namespace-profile-default";

#[test]
fn main_coverage_uses_the_shared_namespace_profile() {
    let workflow = include_str!("../.github/workflows/coverage-main.yml");
    assert!(
        workflow.contains(NAMESPACE_RUNNER),
        "main coverage must use {NAMESPACE_RUNNER}"
    );
}

#[test]
fn ci_uses_a_github_hosted_runner_for_the_whitaker_toolchain() {
    let workflow = include_str!("../.github/workflows/ci.yml");
    assert!(
        workflow.contains("runs-on: ubuntu-latest"),
        "CI requires ubuntu-latest for Whitaker's prebuilt cargo-dylint"
    );
}
