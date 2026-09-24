//! Guard Gauss's reviewed Linux runner assignments.

use rstest::rstest;

/// The coverage publisher's runner. The estate left Namespace on 2026-09-03,
/// so the former `namespace-profile-default` label would queue the publisher
/// forever; the job is single-process coverage, so the smallest Ubicloud size
/// serves it.
const PUBLISHER_RUNNER: &str = "ubicloud-standard-2";

/// Return the YAML block for one named workflow job.
///
/// The block ends at the next sibling key, indented exactly two spaces;
/// deeper-indented lines, including nested blocks before `runs-on`, belong to
/// the job.
fn job_block<'workflow>(workflow: &'workflow str, job_name: &str) -> Option<&'workflow str> {
    let marker = format!("\n  {job_name}:\n");
    let body = workflow.split_once(&marker)?.1;
    let end = body
        .match_indices("\n  ")
        .find(|&(index, _)| {
            body.get(index + 3..)
                .is_some_and(|rest| !rest.starts_with(' '))
        })
        .map_or(body.len(), |(index, _)| index);
    body.get(..end)
}

/// Return whether a job's `runs-on` is exactly one plain label.
///
/// Compared whole: another label, or the sequence or mapping form carrying the
/// same label, does not match.
fn job_runs_on(workflow: &str, job_name: &str, label: &str) -> bool {
    job_block(workflow, job_name).is_some_and(|job| {
        job.lines()
            .any(|line| line.trim() == format!("runs-on: {label}"))
    })
}

#[test]
fn main_coverage_uses_the_ubicloud_publisher_runner() {
    let workflow = include_str!("../.github/workflows/coverage-main.yml");
    assert!(
        job_runs_on(workflow, "coverage-upload", PUBLISHER_RUNNER),
        "coverage-upload must use {PUBLISHER_RUNNER}"
    );
}

/// The label is held by name, so the retired Namespace label, a larger size,
/// or another form of `runs-on` is refused rather than read as compliant.
#[rstest]
#[case::namespace("namespace-profile-default")]
#[case::larger_size("ubicloud-standard-8")]
#[case::hosted("ubuntu-latest")]
#[case::sequence_form("[self-hosted, ubicloud-standard-2]")]
fn publisher_label_is_matched_by_name(#[case] runs_on: &str) {
    let workflow = format!("jobs:\n  coverage-upload:\n    runs-on: {runs_on}\n");
    assert!(
        !job_runs_on(&workflow, "coverage-upload", PUBLISHER_RUNNER),
        "{runs_on} was accepted as {PUBLISHER_RUNNER}"
    );
}

/// Keys before `runs-on`, nested blocks included, stay inside the job, and the
/// next job's keys stay outside it.
#[rstest]
#[case::name_first("    name: Publish\n    runs-on: ubicloud-standard-2\n", true)]
#[case::nested_block_first(
    "    permissions:\n      contents: read\n    runs-on: ubicloud-standard-2\n",
    true
)]
#[case::label_in_next_job(
    "    name: Publish\n  other:\n    runs-on: ubicloud-standard-2\n",
    false
)]
fn job_block_ends_at_the_next_job(#[case] job: &str, #[case] expected: bool) {
    let workflow = format!("jobs:\n  coverage-upload:\n{job}");
    assert_eq!(
        job_runs_on(&workflow, "coverage-upload", PUBLISHER_RUNNER),
        expected,
        "{job}"
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
