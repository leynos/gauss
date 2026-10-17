//! Specifies reusable-call closure, non-vacuity and installer-locality cases.

use rstest::rstest;

use super::super::ACTION_REVISION;
use super::{INSTALLED, fixture_inventory, replace_once};

/// A local reusable workflow is inspected transitively, with its own installer boundary.
#[test]
fn local_reusable_suite_is_inspected() {
    let invoking_workflow = include_str!("fixtures/ci_reachability/discovery-local-reusable.yml");
    let reusable_source = replace_once(INSTALLED, "on: push", "on: workflow_call")
        .expect("callee trigger is an explicit fixture change");
    let baseline = fixture_inventory(&[
        ("caller.yml", invoking_workflow),
        ("discovery-linux-installed.yaml", &reusable_source),
    ])
    .expect("a local reusable suite with its own installer must pass");
    assert_eq!(baseline.suite_jobs, 1);
    let mutant = replace_once(
        &reusable_source,
        "install-mold: true",
        "install-mold: false",
    )
    .expect("one callee installer must change");
    let error = fixture_inventory(&[
        ("caller.yml", invoking_workflow),
        ("discovery-linux-installed.yaml", &mutant),
    ])
    .expect_err("a caller installer cannot replace the callee's own installer");
    assert!(
        error.contains("no unconditional pinned `mold` installer"),
        "{error}"
    );
}

/// The reviewed Dependabot call is an exact non-suite exception, not a shared-actions wildcard.
#[test]
fn dependabot_exception_is_exact() {
    let source = include_str!("fixtures/ci_reachability/discovery-dependabot-approved.yml");
    let baseline = fixture_inventory(&[("suite.yml", INSTALLED), ("bot.yml", source)])
        .expect("the reviewed non-suite call must coexist with the suite inventory");
    assert_eq!(baseline.suite_jobs, 1);
    let mutant = replace_once(
        source,
        ACTION_REVISION,
        "0000000000000000000000000000000000000000",
    )
    .expect("one actual adopted action pin must change");
    let error = fixture_inventory(&[("suite.yml", INSTALLED), ("bot.yml", &mutant)])
        .expect_err("an unreviewed reusable revision must not inherit the exception");
    assert!(error.contains("unprovable reusable workflow"), "{error}");
}

/// Missing local targets and opaque remote or qualified-self calls are explicit measurement gaps.
#[rstest]
#[case::missing_local("./.github/workflows/missing.yml", "missing reusable workflow")]
#[case::qualified_self(
    "leynos/gauss/.github/workflows/ci.yml@main",
    "unprovable reusable workflow"
)]
#[case::unknown_remote(
    "other/repo/.github/workflows/test.yml@main",
    "unprovable reusable workflow"
)]
fn unresolved_reusable_calls_fail_closed(#[case] reference: &str, #[case] diagnostic: &str) {
    let source = format!("on: push\njobs:\n  caller:\n    uses: {reference}\n");
    let error = fixture_inventory(&[("suite.yml", INSTALLED), ("caller.yml", &source)])
        .expect_err("unresolved reusable calls must report a gap");
    assert!(error.contains(diagnostic), "{error}");
}

/// Deleting all workflows or their expected suites must not turn the binding contract green.
#[test]
fn empty_inventory_and_missing_suites_are_rejected() {
    let empty = fixture_inventory(&[]).expect_err("an empty workflow directory must fail");
    assert!(empty.contains("workflow inventory is empty"), "{empty}");
    let jobs = include_str!("fixtures/ci_reachability/empty-jobs.yml");
    let no_jobs = fixture_inventory(&[("empty.yml", jobs)]).expect_err("empty jobs must fail");
    assert!(no_jobs.contains("jobs is empty"), "{no_jobs}");
    let bot = include_str!("fixtures/ci_reachability/discovery-dependabot-approved.yml");
    let no_suite =
        fixture_inventory(&[("bot.yml", bot)]).expect_err("non-suite-only inventory must fail");
    assert!(no_suite.contains("no resolved suite jobs"), "{no_suite}");
}

/// A callee's presence alone cannot prove that GitHub can reach it as a reusable workflow.
#[test]
fn reusable_callee_requires_callable_trigger() {
    let caller = include_str!("fixtures/ci_reachability/discovery-local-reusable.yml");
    let callable = replace_once(INSTALLED, "on: push", "on: workflow_call")
        .expect("the successful reusable baseline must declare its trigger");
    let baseline = fixture_inventory(&[
        ("caller.yml", caller),
        ("discovery-linux-installed.yaml", &callable),
    ])
    .expect("the callable baseline must pass");
    assert_eq!(baseline.suite_jobs, 1);
    let reason = fixture_inventory(&[
        ("caller.yml", caller),
        ("discovery-linux-installed.yaml", INSTALLED),
    ])
    .expect_err("a non-callable workflow is an unprovable reusable route");
    assert!(
        reason.contains("does not declare workflow_call"),
        "{reason}"
    );
}

/// Recursive reusable calls must fail explicitly rather than recurse forever or pass vacuously.
#[test]
fn reusable_cycle_is_rejected() {
    let caller = "on: workflow_call\njobs:\n  caller:\n    uses: ./.github/workflows/caller.yml\n";
    let reason = fixture_inventory(&[("suite.yml", INSTALLED), ("caller.yml", caller)])
        .expect_err("a reusable cycle must be reported as a measurement gap");
    assert!(reason.contains("reusable workflow cycle"), "{reason}");
}

/// A valid installation in a different job does not provision the suite's runner.
#[test]
fn installer_in_other_job_does_not_count() {
    let baseline =
        fixture_inventory(&[("suite.yml", INSTALLED)]).expect("same-job baseline must pass");
    assert_eq!(baseline.linux_rows, 1);
    let no_installer = include_str!("fixtures/ci_reachability/discovery-linux-no-installer.yml");
    let setup_job = concat!(
        "  setup-only:\n    runs-on: ubuntu-latest\n    steps:\n",
        "      - uses: leynos/shared-actions/.github/actions/setup-rust@",
        "e693d81d176d2305e2b736a0be0f9038085dec7f\n",
        "        with:\n          install-mold: true\n"
    );
    let mutated = format!("{no_installer}{setup_job}");
    let reason = fixture_inventory(&[("suite.yml", &mutated)])
        .expect_err("a different runner's installer must not count");
    assert!(
        reason.contains("no unconditional pinned `mold` installer"),
        "{reason}"
    );
}

/// An uncalled callable-only workflow cannot manufacture an event-reachable suite inventory.
#[rstest]
#[case::orphan_only(false)]
#[case::orphan_beside_non_suite_root(true)]
fn orphan_reusable_suite_does_not_satisfy_non_vacuity(#[case] with_bot: bool) {
    let caller = include_str!("fixtures/ci_reachability/discovery-local-reusable.yml");
    let callee = replace_once(INSTALLED, "on: push", "on: workflow_call")
        .expect("one trigger must become callable-only");
    let baseline = fixture_inventory(&[
        ("caller.yml", caller),
        ("discovery-linux-installed.yaml", &callee),
    ])
    .expect("the event-root caller must make its callee reachable");
    assert_eq!(baseline.suite_jobs, 1);
    let mut sources = vec![("discovery-linux-installed.yaml", callee.as_str())];
    if with_bot {
        sources.push((
            "bot.yml",
            include_str!("fixtures/ci_reachability/discovery-dependabot-approved.yml"),
        ));
    }
    let reason = fixture_inventory(&sources)
        .expect_err("removing the only suite caller must not leave a passing orphan inventory");
    assert!(
        reason.contains("no resolved suite jobs reachable from an event"),
        "{reason}"
    );
}

/// Chained workflow triggers need a separate reachability proof rather than an assumed entry root.
#[test]
fn unresolved_workflow_run_trigger_reports_gap() {
    let source = replace_once(INSTALLED, "on: push", "on: workflow_run")
        .expect("one trigger must become an unresolved chain");
    let reason = fixture_inventory(&[("suite.yml", &source)])
        .expect_err("this bounded reader does not claim workflow_run-chain resolution");
    assert!(
        reason.contains("workflow_run chain reachability is not resolved"),
        "{reason}"
    );
}
