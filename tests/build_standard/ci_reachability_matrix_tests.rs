//! Exercises finite runner-platform and one-axis matrix resolution.

use std::collections::BTreeMap;

use super::{Host, Value, matrix, parser, runner};

/// Keeps unsupported matrix forms beside their positive finite-row cases.
#[cfg(test)]
#[path = "ci_reachability_matrix_rejection_tests.rs"]
mod rejections;

/// Parses one fixture's build job, reporting every missing structural layer.
fn fixture_job(source: &str) -> Result<Value, String> {
    let document = parser::parse(source)?;
    let jobs = document
        .required("jobs", "fixture workflow")?
        .mapping("jobs")?;
    jobs.get("build")
        .cloned()
        .ok_or_else(|| "fixture workflow is missing the build job".to_owned())
}

/// Resolves every runner row in one workflow fixture without filtering unknown rows.
fn fixture_hosts(source: &str) -> Result<Vec<Host>, String> {
    runner::hosts(&fixture_job(source)?)
}

/// Expands a supplied matrix body in a minimal workflow with a matrix runner expression.
fn matrix_rows(body: &str) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut source = String::from(concat!(
        "on: pull_request\njobs:\n  build:\n    runs-on: ${{ matrix.runner }}\n",
        "    strategy:\n      matrix:\n"
    ));
    source.push_str(body);
    source.push_str("\n    steps:\n      - run: make test-ci\n");
    let job = fixture_job(&source)?;
    matrix::rows(job.field("strategy"))
}

/// Requires a parser or matrix result to fail with the diagnostic tied to its unsupported form.
fn error_contains<T>(result: Result<T, String>, expected: &str) -> Result<(), String> {
    match result {
        Ok(_) => Err(format!("expected an error containing `{expected}`")),
        Err(error) if error.contains(expected) => Ok(()),
        Err(error) => Err(format!("expected `{expected}`, received `{error}`")),
    }
}

/// Compares one fallible fixture observation at the original named test's assertion boundary.
/// Source and expected hosts are evaluated once, in lookup-before-comparison order.
macro_rules! assert_fixture_hosts {
    ($source:expr, $expected:expr, $setup:literal, $reason:literal) => {{
        let hosts = fixture_hosts($source).expect($setup);
        assert_eq!(hosts, $expected, $reason);
    }};
}

/// Reads the scalar runner from the Linux scalar fixture.
#[test]
fn runner_linux_scalar_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/linux-scalar.yml"),
        vec![Host::Linux],
        "the Linux scalar fixture must parse and resolve",
        "Ubuntu runner proves Linux"
    );
}

/// Recognizes only the exact Ubicloud standard label as Linux evidence.
#[test]
fn runner_ubicloud_standard_two_label() {
    let source = "on: pull_request\njobs:\n  build:\n    runs-on: ubicloud-standard-2\n";
    assert_eq!(
        fixture_hosts(source).expect("the exact Ubicloud label must resolve"),
        vec![Host::Linux],
        "exact Ubicloud label proves Linux"
    );
    let lookalike = source.replace("ubicloud-standard-2", "ubicloud-standard-2-extra");
    error_contains(
        fixture_hosts(&lookalike),
        "do not prove a supported operating system",
    )
    .expect("the Ubicloud lookalike must not prove Linux");
}

/// Reads inline runner labels as one required runner label set.
#[test]
fn runner_linux_inline_list_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/linux-inline-list.yml"),
        vec![Host::Linux],
        "the inline Linux list fixture must parse and resolve",
        "inline Linux label proves the host"
    );
}

/// Reads block runner labels without treating list entries as alternative runners.
#[test]
fn runner_linux_block_list_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/linux-block-list.yml"),
        vec![Host::Linux],
        "the block Linux list fixture must parse and resolve",
        "block Linux label proves the host"
    );
}

/// Uses labels, rather than the opaque group name, to prove a grouped Linux runner.
#[test]
fn runner_linux_group_labels_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/linux-group-labels.yml"),
        vec![Host::Linux],
        "the grouped Linux fixture must parse and resolve",
        "group labels prove Linux"
    );
}

/// Preserves macOS as a real non-Linux runner row.
#[test]
fn runner_macos_inline_list_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/macos-inline-list.yml"),
        vec![Host::Darwin],
        "the macOS list fixture must parse and resolve",
        "macOS label remains non-Linux"
    );
}

/// Expands each scalar axis row, retaining the Linux and non-Linux cases in source order.
#[test]
fn matrix_scalar_mixed_hosts_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/matrix-scalar-mixed.yml"),
        vec![Host::Linux, Host::Darwin, Host::Windows],
        "the mixed scalar matrix fixture must parse and resolve",
        "all matrix hosts remain"
    );
}

/// Retains a matrix containing only legitimate non-Linux rows.
#[test]
fn matrix_all_nonlinux_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/matrix-all-nonlinux-block.yml"),
        vec![Host::Darwin, Host::Windows],
        "the all-non-Linux matrix fixture must parse and resolve",
        "non-Linux rows are not discarded"
    );
}

/// Applies scalar include and exclude rows without widening the finite matrix.
#[test]
fn matrix_include_exclude_fixture() {
    let source = include_str!("fixtures/ci_reachability/matrix-block-map-include-exclude.yml");
    let hosts = fixture_hosts(source).expect("the include/exclude fixture must parse and resolve");
    assert_eq!(
        hosts,
        vec![Host::Linux, Host::Windows],
        "excluded macOS row is removed"
    );
    let job = fixture_job(source).expect("the include/exclude job must parse");
    let rows = matrix::rows(job.field("strategy")).expect("the scalar matrix must expand");
    let profile = rows
        .first()
        .and_then(|row| row.get("profile").map(String::as_str));
    assert_eq!(profile, Some("debug"), "include metadata merges");
}

/// Resolves a finite include-only matrix without requiring a base axis.
#[test]
fn matrix_include_only_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/matrix-include-only.yml"),
        vec![Host::Linux, Host::Windows],
        "the include-only fixture must parse and resolve",
        "both explicit rows remain"
    );
}

/// Substitutes an exact matrix field recursively in an inline runner label list.
#[test]
fn matrix_list_expression_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/matrix-list-expression.yml"),
        vec![Host::Linux, Host::Windows],
        "the list-expression matrix fixture must parse and resolve",
        "list expressions resolve per row"
    );
}

/// Substitutes an exact matrix field in the labels member of a runner mapping.
#[test]
fn matrix_mapping_expression_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/matrix-map-expression.yml"),
        vec![Host::Linux, Host::Darwin],
        "the mapping-expression matrix fixture must parse and resolve",
        "mapping expressions resolve per row"
    );
}

/// Resolves a Windows runner mapping with inline group and labels.
#[test]
fn runner_windows_inline_mapping_fixture() {
    assert_fixture_hosts!(
        include_str!("fixtures/ci_reachability/windows-inline-map.yml"),
        vec![Host::Windows],
        "the Windows runner mapping fixture must parse and resolve",
        "Windows label proves the host"
    );
}

/// Resolves a labels-only runner mapping without requiring an opaque group name.
#[test]
fn runner_labels_only_mapping_resolves_os() {
    let source = "on: push\njobs:\n  build:\n    runs-on: {labels: ubuntu-latest}\n";
    assert_eq!(
        fixture_hosts(source).expect("literal labels-only mapping must resolve"),
        vec![Host::Linux],
        "labels establish the runner operating system"
    );
}
