//! Inventory, reusable-call and non-vacuity regression cases for suite provisioning.

use rstest::rstest;

use super::super::rust_flags::replace_once;
use super::tests::fixture_inventory;
use super::{Coverage, Value, parser};

/// Keeps reusable closure and non-vacuity cases together.
#[cfg(test)]
#[path = "ci_reachability_workflows_reuse_tests.rs"]
mod reuse_cases;

/// The known installed workflow used to prove every mutation starts from a successful inventory.
const INSTALLED: &str = include_str!("fixtures/ci_reachability/discovery-ci.yml");

/// Both filename extensions are discovered, including an entirely new suite workflow.
#[rstest]
#[case::new_yml("added.yml")]
#[case::new_yaml("added.yaml")]
fn mutation_new_workflow_is_detected(#[case] name: &str) {
    let baseline = fixture_inventory(&[("existing.yml", INSTALLED)])
        .expect("baseline workflow must pass before adding a new source");
    assert_eq!(baseline.suite_jobs, 1);
    let missing = include_str!("fixtures/ci_reachability/discovery-linux-no-installer.yml");
    let error = fixture_inventory(&[("existing.yml", INSTALLED), (name, missing)])
        .expect_err("the newly discovered Linux suite requires its own installer");
    assert!(
        error.contains("no unconditional pinned `mold` installer"),
        "{error}"
    );
}

/// Coverage actions reach suites even when their RUSTFLAGS exclude the development linker.
#[test]
fn coverage_action_requires_installer_before_suite() {
    let source = include_str!("fixtures/ci_reachability/coverage-action.yml");
    let baseline = fixture_inventory(&[("coverage.yml", source)])
        .expect("installed coverage baseline must pass");
    assert_eq!(
        baseline,
        Coverage {
            suite_jobs: 1,
            linux_rows: 1
        }
    );
    let mutant = replace_once(source, "install-mold: true", "install-mold: false")
        .expect("one coverage installer input must change");
    let error = fixture_inventory(&[("coverage.yml", &mutant)])
        .expect_err("coverage tests still need their current `mold` provisioner");
    assert!(
        error.contains("no unconditional pinned `mold` installer"),
        "{error}"
    );
}

/// macOS and Windows suites remain legitimate exceptions to Linux-only tool installation.
#[rstest]
#[case::macos("macos-latest")]
#[case::windows("windows-latest")]
fn non_linux_suite_needs_no_mold(#[case] label: &str) {
    let missing = include_str!("fixtures/ci_reachability/discovery-linux-no-installer.yml");
    let source =
        replace_once(missing, "ubuntu-latest", label).expect("one runner label must be replaced");
    let coverage = fixture_inventory(&[("other.yml", &source)])
        .expect("non-Linux suites must not acquire a Linux installer requirement");
    assert_eq!(
        coverage,
        Coverage {
            suite_jobs: 1,
            linux_rows: 0
        }
    );
}

/// Every supplied workflow fixture has an explicit passing or failing installation verdict.
#[rstest]
#[case::discovery_coverage(include_str!("fixtures/ci_reachability/discovery-coverage.yml"), None)]
#[case::discovery_linux(include_str!("fixtures/ci_reachability/discovery-linux-installed.yaml"), None)]
#[case::late_installer(include_str!("fixtures/ci_reachability/late-installer.yml"), Some("no unconditional pinned `mold` installer"))]
fn discovery_fixture_has_specific_verdict(#[case] source: &str, #[case] diagnostic: Option<&str>) {
    let result = fixture_inventory(&[("fixture.yml", source)]);
    match diagnostic {
        None => {
            let coverage = result.expect("installed fixture must pass");
            assert_eq!(coverage.linux_rows, 1);
        }
        Some(expected) => {
            let reason = result.expect_err("the fixture's installation defect must fail");
            assert!(reason.contains(expected), "{reason}");
        }
    }
}

/// Boolean-like trigger spellings remain an explicit ambiguity even beside a valid literal on key.
#[rstest]
#[case::boolean_true("true")]
#[case::boolean_true_capital("True")]
#[case::boolean_false("false")]
fn boolean_trigger_keys_are_rejected(#[case] key: &str) {
    let baseline = fixture_inventory(&[("suite.yml", INSTALLED)])
        .expect("the ordinary literal on trigger must pass");
    assert_eq!(baseline.suite_jobs, 1);
    let mutated = format!("{key}: push\n{INSTALLED}");
    let reason = fixture_inventory(&[("suite.yml", &mutated)])
        .expect_err("a boolean-like trigger key must not be read ambiguously");
    assert!(reason.contains("ambiguous boolean trigger key"), "{reason}");
}

/// Scalar, sequence and mapping event forms retain the same discovered installation requirement.
#[rstest]
#[case::scalar("on: push")]
#[case::sequence("on: [push, pull_request]")]
#[case::mapping("on:\n  push:\n  pull_request:")]
fn trigger_shapes_are_measured(#[case] trigger: &str) {
    let source = replace_once(INSTALLED, "on: push", trigger)
        .expect("one trigger definition must be restated");
    let coverage = fixture_inventory(&[("suite.yml", &source)])
        .expect("a literal supported trigger form must retain its suite");
    assert_eq!(
        coverage,
        Coverage {
            suite_jobs: 1,
            linux_rows: 1
        }
    );
}

/// Empty or computed triggers cannot create an apparently measured workflow with no proven event.
#[rstest]
#[case::empty_sequence("on: []", "expected a mapping")]
#[case::empty_mapping("on: {}", "trigger is empty or indeterminate")]
#[case::computed("on: ${{ inputs.event }}", "unresolved workflow trigger event")]
fn indeterminate_triggers_are_rejected(#[case] trigger: &str, #[case] diagnostic: &str) {
    let baseline = fixture_inventory(&[("suite.yml", INSTALLED)])
        .expect("the literal trigger baseline must pass");
    assert_eq!(baseline.suite_jobs, 1);
    let source = replace_once(INSTALLED, "on: push", trigger).expect("one trigger must change");
    let reason = fixture_inventory(&[("suite.yml", &source)])
        .expect_err("an indeterminate trigger must report its gap");
    assert!(reason.contains(diagnostic), "{reason}");
}

/// Directory, shell and tool-input overrides cannot silently select a different source route.
#[rstest]
#[case::step_directory(
    "      - run:",
    "      - working-directory: nested\n        run:",
    "working-directory"
)]
#[case::job_defaults(
    "    runs-on:",
    "    defaults:\n      run:\n        working-directory: nested\n    runs-on:",
    "working-directory"
)]
#[case::workflow_defaults(
    "on: push\n",
    "on: push\ndefaults:\n  run:\n    shell: python\n",
    "workflow shell"
)]
#[case::step_shell(
    "      - run:",
    "      - shell: ./wrapper {0}\n        run:",
    "workflow shell"
)]
#[case::job_tool_input(
    "    runs-on:",
    "    env:\n      UV: ./wrapper\n    runs-on:",
    "execution environment input: UV"
)]
#[case::job_pytest_input(
    "    runs-on:",
    "    env:\n      PYTEST_ADDOPTS: scripts/unreviewed.py\n    runs-on:",
    "execution environment input: PYTEST_ADDOPTS"
)]
#[case::workflow_make_input(
    "on: push\n",
    "on: push\nenv:\n  MAKEFLAGS: -f hidden.mk\n",
    "execution environment input: MAKEFLAGS"
)]
#[case::job_container(
    "    runs-on:",
    "    container: example/image:latest\n    runs-on:",
    "uninspected job container or service"
)]
#[case::job_services(
    "    runs-on:",
    "    services:\n      dependency:\n        image: example/image:latest\n    runs-on:",
    "uninspected job container or service"
)]
fn execution_boundary_overrides_report_gaps(
    #[case] original: &str,
    #[case] changed: &str,
    #[case] diagnostic: &str,
) {
    let baseline = fixture_inventory(&[("suite.yml", INSTALLED)])
        .expect("the root-relative current source must pass before mutation");
    assert_eq!(baseline.linux_rows, 1);
    let mutant = replace_once(INSTALLED, original, changed)
        .expect("exactly one execution boundary must change");
    let reason = fixture_inventory(&[("suite.yml", &mutant)])
        .expect_err("unresolved execution context must not inherit a source-route verdict");
    assert!(reason.contains(diagnostic), "{reason}");
}

/// A compiler-only environment value retains suite reachability and its installation requirement.
#[test]
fn compiler_environment_keeps_suite_reachability() {
    let source = replace_once(INSTALLED, "on: push\n", "on: push\nenv:\n  RUSTFLAGS: ''\n")
        .expect("one root environment declaration must be inserted");
    let coverage = fixture_inventory(&[("suite.yml", &source)])
        .expect("compiler flags alone do not change the inspected test command");
    assert_eq!(
        coverage,
        Coverage {
            suite_jobs: 1,
            linux_rows: 1
        }
    );
}

/// Reads the build job from one parser fixture without hiding malformed structure.
fn parser_fixture_job(source: &str) -> Result<Value, String> {
    let document = parser::parse(source)?;
    let jobs = document
        .required("jobs", "parser fixture")?
        .mapping("jobs")?;
    jobs.get("build")
        .cloned()
        .ok_or_else(|| "parser fixture is missing the build job".to_owned())
}

/// Reads selected scalar fields from sequence-mapping workflow steps.
fn parser_step_scalars(job: &Value, key: &str) -> Result<Vec<String>, String> {
    let steps = job
        .required("steps", "parser fixture job")?
        .sequence("steps")?;
    let mut values = Vec::new();
    for step in steps {
        step.mapping("workflow step")?;
        if let Some(value) = step.field(key) {
            values.push(value.text(key)?.to_owned());
        }
    }
    Ok(values)
}

/// Keeps hashes in quoted scalars and GitHub expressions inside shell text unchanged.
#[test]
fn parser_quoted_hash_expression_fixture() {
    let source = include_str!("fixtures/ci_reachability/quoted-hash.yml");
    let document = parser::parse(source).expect("the quoted-hash fixture must parse");
    let job = parser_fixture_job(source).expect("the quoted-hash build job must parse");
    let release = job
        .required("env", "job")
        .expect("the job environment must exist")
        .field("RELEASE_NOTE")
        .expect("the quoted release note must exist");
    assert_eq!(
        release
            .text("RELEASE_NOTE")
            .expect("release note is scalar"),
        "release #42"
    );
    let runs = parser_step_scalars(&job, "run").expect("step run values must parse");
    assert!(
        runs.iter()
            .any(|run| run == "echo \"note #42; ${{ github.ref }}\""),
        "quoted shell hash and expression remain verbatim"
    );
    assert!(
        document.field("on").is_some(),
        "trigger spelling remains textual"
    );
}

/// Parses every current workflow and checks literal, folded and token command values exactly.
#[test]
fn current_workflows_preserve_block_and_token_text() {
    let ci = parser::parse(include_str!("../../.github/workflows/ci.yml"))
        .expect("the current CI workflow must parse");
    let coverage = parser::parse(include_str!("../../.github/workflows/coverage-main.yml"))
        .expect("the current coverage workflow must parse");
    let dependabot = parser::parse(include_str!(
        "../../.github/workflows/dependabot-automerge.yml"
    ))
    .expect("the current Dependabot workflow must parse");
    for workflow in [&ci, &coverage, &dependabot] {
        assert!(
            workflow.field("on").is_some(),
            "current workflow has a textual trigger key"
        );
        assert!(
            workflow.field("jobs").is_some(),
            "current workflow jobs are parsed"
        );
    }
    let ci_jobs = ci
        .required("jobs", "ci")
        .expect("CI jobs must exist")
        .mapping("jobs")
        .expect("CI jobs must be a mapping");
    let ci_job = ci_jobs
        .get("build-test")
        .expect("CI build-test job must exist");
    let runs = parser_step_scalars(ci_job, "run").expect("CI step run values must parse");
    assert!(
        runs.iter()
            .any(|run| run.starts_with("sudo apt-get update\nsudo apt-get install")),
        "literal shell block keeps its body indentation and line breaks"
    );
    let uses = parser_step_scalars(ci_job, "uses").expect("CI step actions must parse");
    let setup_rust = concat!(
        "leynos/shared-actions/.github/actions/setup-rust@",
        "e693d81d176d2305e2b736a0be0f9038085dec7f"
    );
    assert!(
        uses.iter().any(|value| value == setup_rust),
        "folded action reference joins into the exact pinned scalar"
    );
    let coverage_jobs = coverage
        .required("jobs", "coverage")
        .expect("coverage jobs must exist")
        .mapping("jobs")
        .expect("coverage jobs must be a mapping");
    let coverage_job = coverage_jobs
        .get("coverage-upload")
        .expect("coverage-upload job must exist");
    let coverage_runs =
        parser_step_scalars(coverage_job, "run").expect("coverage run values must parse");
    assert!(
        coverage_runs.iter().any(|run| run
            == "echo \"available=${{ secrets.CS_ACCESS_TOKEN != '' }}\" >> \"$GITHUB_OUTPUT\""),
        "quoted token-presence command remains verbatim"
    );
}
