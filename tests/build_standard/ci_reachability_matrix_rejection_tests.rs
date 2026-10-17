//! Rejects matrix and runner forms whose finite host meaning is unproven.

use std::collections::BTreeMap;

use rstest::rstest;

use super::{error_contains, fixture_job, matrix_rows, runner};

/// Rejects scalar axes with multiple dimensions, nested values, and dynamic entries.
#[test]
fn matrix_rejects_unsupported_axes() {
    error_contains(
        matrix_rows("        os: [ubuntu-latest]\n        rust: [stable]"),
        "only one matrix axis",
    )
    .expect("multiple matrix axes must be refused");
    error_contains(
        matrix_rows("        os: [[ubuntu-latest]]"),
        "expected a scalar",
    )
    .expect("nested matrix values must be refused");
    error_contains(
        matrix_rows("        os: [${{ github.runner }}]"),
        "computed",
    )
    .expect("computed matrix values must be refused");
    error_contains(
        matrix_rows("        ${{ github.axis }}: [ubuntu-latest]"),
        "computed matrix axis name",
    )
    .expect("computed matrix keys must be refused");
    error_contains(matrix_rows("        os: []"), "empty")
        .expect("empty matrix axes must be refused");
}

/// Rejects empty finite expansions and scalar include/exclude rows with unsupported structure.
#[test]
fn matrix_rejects_empty_and_nested_rows() {
    error_contains(matrix_rows("        include: []"), "empty")
        .expect("empty include-only matrices must be refused");
    error_contains(
        matrix_rows(concat!(
            "        os: [ubuntu-latest]\n",
            "        exclude:\n          - os: ubuntu-latest"
        )),
        "expansion is empty",
    )
    .expect("an exclusion that removes every row must be refused");
    error_contains(
        matrix_rows("        include:\n          - runner: {os: ubuntu-latest}"),
        "expected a scalar",
    )
    .expect("nested include values must be refused");
    error_contains(
        matrix_rows("        include:\n          - ${{ github.key }}: ubuntu-latest"),
        "computed key",
    )
    .expect("computed include keys must be refused");
}

/// Rejects include conflicts and exclusions that name values outside the sole scalar axis.
#[test]
fn matrix_rejects_conflicting_metadata() {
    error_contains(
        matrix_rows(concat!(
            "        os: [ubuntu-latest]\n",
            "        include:\n          - os: ubuntu-latest\n            profile: debug\n",
            "          - os: ubuntu-latest\n            profile: release"
        )),
        "conflicts",
    )
    .expect("conflicting include metadata must be refused");
    error_contains(
        matrix_rows(concat!(
            "        os: [ubuntu-latest]\n",
            "        exclude:\n          - os: ubuntu-latest\n            profile: debug"
        )),
        "only the scalar axis",
    )
    .expect("exclusion metadata beyond the scalar axis must be refused");
    error_contains(
        matrix_rows(concat!(
            "        os: [ubuntu-latest]\n",
            "        include:\n          - profile: debug"
        )),
        "must name `os`",
    )
    .expect("metadata-only include rows are outside the supported matrix subset");
}

/// Rejects missing, partial, and computed matrix expressions in runner values.
#[test]
fn matrix_rejects_unresolved_runner_expressions() {
    let missing = concat!(
        "on: pull_request\njobs:\n  build:\n",
        "    runs-on: ${{ matrix.runner }}\n    strategy:\n      matrix:\n",
        "        os: [ubuntu-latest]\n"
    );
    let job = fixture_job(missing).expect("the missing-field matrix must parse");
    error_contains(runner::hosts(&job), "missing from the row")
        .expect("an absent matrix field must be refused");
    let partial = "on: pull_request\njobs:\n  build:\n    runs-on: prefix-${{ matrix.os }}\n";
    let partial_job = fixture_job(partial).expect("the partial expression matrix must parse");
    error_contains(runner::hosts(&partial_job), "unsupported computed")
        .expect("a partial matrix expression must be refused");
}

/// Rejects runner groups without OS labels, empty labels, computed labels, and conflicting OSes.
#[test]
fn runner_rejects_unprovable_and_conflicting_labels() {
    for (source, expected) in [
        (
            "on: pull_request\njobs:\n  build:\n    runs-on: self-hosted\n",
            "do not prove a supported operating system",
        ),
        (
            "on: pull_request\njobs:\n  build:\n    runs-on: {group: pool}\n",
            "missing `labels`",
        ),
        (
            "on: pull_request\njobs:\n  build:\n    runs-on: {group: pool, labels: []}\n",
            "runs-on label list is empty",
        ),
        (
            concat!(
                "on: pull_request\njobs:\n  build:\n    runs-on: {group: pool, labels: ",
                "[ubuntu-latest], extra: value}\n"
            ),
            "may contain only `group` and `labels`",
        ),
        (
            "on: pull_request\njobs:\n  build:\n    runs-on: \"\"\n",
            "runs-on label is empty or computed",
        ),
        (
            "on: pull_request\njobs:\n  build:\n    runs-on: ${{ github.runner }}\n",
            "unsupported computed runner expression",
        ),
        (
            "on: pull_request\njobs:\n  build:\n    runs-on: [ubuntu-latest, windows-latest]\n",
            "conflicting operating systems",
        ),
    ] {
        let job = fixture_job(source).expect("the runner mutation must parse");
        error_contains(runner::hosts(&job), expected)
            .expect("the runner mutation must be rejected for its stated reason");
    }
}

/// Baseline and rejection diagnostics for one repeated matrix-row scenario.
struct DuplicateRowProof {
    /// Context if the unique baseline cannot expand.
    baseline_setup: &'static str,
    /// Reason the complete unique baseline rows must remain supported.
    baseline_assertion: &'static str,
    /// Context if the repeated-row mutation unexpectedly expands.
    mutation_reason: &'static str,
    /// Exact include or exclude diagnostic required from the mutation.
    expected_error: &'static str,
}

/// Proves each complete unique baseline before rejecting its exact repeated row.
#[rstest]
#[case::duplicate_include_only(
    concat!(
        "        include:\n",
        "          - runner: ubuntu-latest\n",
        "          - runner: windows-latest",
    ),
    &[
        BTreeMap::from([("runner".to_owned(), "ubuntu-latest".to_owned())]),
        BTreeMap::from([("runner".to_owned(), "windows-latest".to_owned())]),
    ],
    "\n          - runner: ubuntu-latest",
    &DuplicateRowProof {
        baseline_setup: "unique include-only rows must expand",
        baseline_assertion: "unique include-only rows remain supported",
        mutation_reason: "a repeated include-only row is unsupported",
        expected_error: "matrix.include: repeated scalar rows are unsupported by this reader",
    }
)]
#[case::duplicate_axis_include(
    concat!(
        "        os: [ubuntu-latest]\n",
        "        include:\n          - os: windows-latest",
    ),
    &[
        BTreeMap::from([("os".to_owned(), "ubuntu-latest".to_owned())]),
        BTreeMap::from([("os".to_owned(), "windows-latest".to_owned())]),
    ],
    "\n          - os: windows-latest",
    &DuplicateRowProof {
        baseline_setup: "a unique axis include must expand",
        baseline_assertion: "unique axis include rows remain supported",
        mutation_reason: "a repeated axis include is unsupported",
        expected_error: "matrix.include: repeated scalar rows are unsupported by this reader",
    }
)]
#[case::duplicate_axis_exclude(
    concat!(
        "        os: [ubuntu-latest, windows-latest, macos-latest]\n",
        "        exclude:\n          - os: macos-latest",
    ),
    &[
        BTreeMap::from([("os".to_owned(), "ubuntu-latest".to_owned())]),
        BTreeMap::from([("os".to_owned(), "windows-latest".to_owned())]),
    ],
    "\n          - os: macos-latest",
    &DuplicateRowProof {
        baseline_setup: "a unique axis exclusion must expand",
        baseline_assertion: "unique axis exclusion rows remain supported",
        mutation_reason: "a repeated axis exclusion is unsupported",
        expected_error: "matrix.exclude: repeated scalar rows are unsupported by this reader",
    }
)]
fn matrix_rejects_repeated_rows(
    #[case] unique: &str,
    #[case] expected: &[BTreeMap<String, String>],
    #[case] suffix: &str,
    #[case] proof: &DuplicateRowProof,
) {
    let baseline_rows = matrix_rows(unique).expect(proof.baseline_setup);
    assert_eq!(
        baseline_rows.as_slice(),
        expected,
        "{}",
        proof.baseline_assertion
    );
    let repeated = [unique, suffix].concat();
    let error = matrix_rows(&repeated).expect_err(proof.mutation_reason);
    assert_eq!(error, proof.expected_error);
}
