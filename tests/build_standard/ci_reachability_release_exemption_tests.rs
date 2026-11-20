//! Cases pinning the release-workflow exemption's scope and fail-closed behaviour.

use rstest::rstest;

use super::super::super::tests::fixture_inventory;

/// The known installed workflow, used to keep every probe inventory non-vacuous.
const INSTALLED: &str = include_str!("fixtures/ci_reachability/discovery-ci.yml");

/// A release workflow may build and package, but the exemption must not hide a suite.
///
/// The exemption exists because release steps call packaging actions the build
/// standard never reviewed. It must still fail closed: a suite added to an
/// exempt path is a contract failure, not something the skip swallows.
#[rstest]
#[case::suite_command("run: cargo test --workspace")]
#[case::nextest_command("run: cargo nextest run --workspace")]
#[case::coverage_action(concat!(
    "uses: leynos/shared-actions/.github/actions/generate-coverage@",
    "6e99f225cb06684f9e6f89b63a7ca4a04095c067"
))]
fn exempt_release_workflow_still_rejects_a_suite(#[case] step: &str) {
    let source = format!(
        "on: workflow_call\njobs:\n\x20 build:\n\x20   runs-on: ubuntu-latest\n\x20   \
         steps:\n\x20     - {step}\n"
    );
    // The baseline suite keeps the inventory non-vacuous, so the failure below
    // can only be the exempt path's own suite check.
    let error = fixture_inventory(&[("suite.yml", INSTALLED), ("build-and-package.yml", &source)])
        .expect_err("a suite in an exempt release path must fail the contract");
    assert!(
        error.contains("must not run a test suite"),
        "wrong diagnostic: {error}"
    );
}

/// An exempt workflow still has to prove which operating system each job uses.
///
/// The exemption widens the action vocabulary, not the runner-provenance rule,
/// so a runner computed from an input remains unprovable here too.
#[test]
fn exempt_release_workflow_still_requires_a_provable_runner() {
    let source = concat!(
        "on: workflow_call\n",
        "jobs:\n",
        "  build:\n",
        "    runs-on: ${{ inputs.runner }}\n",
        "    steps:\n",
        "      - run: echo building\n"
    );
    let error = fixture_inventory(&[("suite.yml", INSTALLED), ("build-and-package.yml", source)])
        .expect_err("a computed runner must stay unprovable in an exempt workflow");
    assert!(
        error.contains("unsupported computed runner expression"),
        "wrong diagnostic: {error}"
    );
}

/// The exemption is keyed to exact paths, so any other workflow stays fully inspected.
///
/// A packaging-shaped job placed anywhere else must still satisfy the ordinary
/// suite contract, which proves the exemption cannot be inherited by a new file.
#[test]
fn non_exempt_workflow_is_still_fully_inspected() {
    let packaging_shaped = concat!(
        "on: push\n",
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: cargo nextest run --workspace\n"
    );
    let error = fixture_inventory(&[
        ("suite.yml", INSTALLED),
        ("renamed-package.yml", packaging_shaped),
    ])
    .expect_err("an unreviewed path must not inherit the release exemption");
    assert!(
        error.contains("no unconditional pinned `mold` installer"),
        "the ordinary suite contract must still apply: {error}"
    );
}
