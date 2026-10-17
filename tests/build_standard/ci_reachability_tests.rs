//! Binding production and installation mutations for the discovered suite contract.

use camino::Utf8Path;
use cap_std::{ambient_authority, fs_utf8::Dir};
use rstest::rstest;
use uuid::Uuid;

use super::super::rust_flags::replace_once;
use super::{Coverage, Workspace, inspect};

/// Keeps strict YAML scalar and block-body regression cases with the parser boundary.
#[cfg(test)]
#[path = "ci_reachability_parser_tests.rs"]
mod parser_cases;

/// Keeps Make source-safety mutations beside their explicit parser-level oracle.
#[cfg(test)]
#[path = "ci_reachability_root_source_tests.rs"]
mod source_cases;

/// The successful current-installer fixture, kept external so mutations target real YAML structure.
const INSTALLED: &str = include_str!("fixtures/ci_reachability/discovery-ci.yml");
/// The exact step removed or moved by installation-order mutations.
const INSTALLER: &str = concat!(
    "      - uses: leynos/shared-actions/.github/actions/setup-rust@",
    "e693d81d176d2305e2b736a0be0f9038085dec7f\n",
    "        with:\n          install-mold: true\n"
);

/// Opens the explicit repository capability boundary used by source-reading contracts.
pub(super) fn project_directory() -> Result<Dir, String> {
    Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .map_err(|error| format!("opening project capability: {error}"))
}

/// Stages an owned inventory under the ignored target directory and always removes its mutations.
///
/// For example, a single installed Linux workflow yields one suite job and one Linux row.
pub(super) fn fixture_inventory(sources: &[(&str, &str)]) -> Result<Coverage, String> {
    with_fixture(sources, inspect)
}

/// Runs an operation in an exclusive capability-rooted fixture, cleaning up on either result.
pub(super) fn with_fixture<T>(
    sources: &[(&str, &str)],
    operation: impl FnOnce(&Workspace<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let directory = project_directory()?;
    let relative = format!("target/reachability-{}", Uuid::new_v4());
    directory
        .create_dir_all("target")
        .map_err(|error| format!("creating fixture parent: {error}"))?;
    directory
        .create_dir(&relative)
        .map_err(|error| format!("creating exclusive workflow fixture: {error}"))?;
    let result = staged_inventory(&directory, &relative, sources, operation);
    let cleanup = directory
        .remove_dir_all(&relative)
        .map_err(|error| format!("removing workflow fixture: {error}"));
    cleanup?;
    result
}

/// Writes only named workflow inputs into a fresh owned root before inspecting that complete root.
fn staged_inventory<T>(
    parent: &Dir,
    relative: &str,
    sources: &[(&str, &str)],
    operation: impl FnOnce(&Workspace<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let directory = parent
        .open_dir(relative)
        .map_err(|error| format!("opening fixture root: {error}"))?;
    directory
        .create_dir_all(".github/workflows")
        .map_err(|error| format!("creating workflow fixture inventory: {error}"))?;
    directory
        .write("Makefile", include_str!("../../Makefile"))
        .map_err(|error| format!("writing fixture Makefile: {error}"))?;
    let mut names = std::collections::BTreeSet::new();
    for (name, source) in sources {
        if !names.insert(*name) {
            return Err(format!("duplicate fixture workflow name: {name}"));
        }
        if contains_path_separator(name) || name.contains("..") {
            return Err(format!("invalid fixture workflow name: {name}"));
        }
        directory
            .write(format!(".github/workflows/{name}"), source)
            .map_err(|error| format!("writing fixture workflow: {error}"))?;
    }
    let path = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    operation(&Workspace {
        directory: &directory,
        path: &path,
    })
}

/// Recognizes both platform path separators in a fixture workflow name.
///
/// For example, `suite.yml` has no separator, while `nested/suite.yml` and
/// `nested\\suite.yml` do.
fn contains_path_separator(name: &str) -> bool {
    name.contains('/') || name.contains('\\')
}

/// The real repository must expose a non-empty measured suite topology.
#[test]
fn repository_suite_jobs_install_pinned_mold_before_execution() -> Result<(), String> {
    let directory = project_directory()?;
    let coverage = inspect(&Workspace {
        directory: &directory,
        path: Utf8Path::new(env!("CARGO_MANIFEST_DIR")),
    })?;
    if coverage.suite_jobs == 0 || coverage.linux_rows == 0 {
        return Err(format!("repository suite coverage is empty: {coverage:?}"));
    }
    Ok(())
}

/// Installation mutations start from a passing baseline and each require its specific rejection.
#[rstest]
#[case::missing_installer(INSTALLER, "", "no unconditional pinned `mold` installer")]
#[case::conditional_installer(
    "        with:\n",
    "        if: runner.os == 'Linux'\n        with:\n",
    "`mold` installer must be unconditional"
)]
#[case::soft_installer(
    "        with:\n",
    "        continue-on-error: false\n        with:\n",
    "`mold` installer must be unconditional"
)]
#[case::disabled_installer(
    "install-mold: true",
    "install-mold: false",
    "no unconditional pinned `mold` installer"
)]
#[case::wrong_installer_pin(
    "setup-rust@e693d81d176d2305e2b736a0be0f9038085dec7f",
    "setup-rust@0000000000000000000000000000000000000000",
    "unprovable action reference"
)]
#[case::conditional_suite(
    "      - run:",
    "      - if: success()\n        run:",
    "suite step must be unconditional"
)]
#[case::soft_suite(
    "      - run:",
    "      - continue-on-error: true\n        run:",
    "suite step must be unconditional"
)]
#[case::soft_suite_job(
    "    runs-on:",
    "    continue-on-error: true\n    runs-on:",
    "suite job must propagate failure"
)]
fn installation_mutations_are_detected(
    #[case] from: &str,
    #[case] to: &str,
    #[case] diagnostic: &str,
) {
    let baseline = fixture_inventory(&[("suite.yml", INSTALLED)])
        .expect("the current pinned installer baseline must be measurable");
    assert_eq!(
        baseline,
        Coverage {
            suite_jobs: 1,
            linux_rows: 1
        }
    );
    let mutated =
        replace_once(INSTALLED, from, to).expect("mutation must have one exact source match");
    let error = fixture_inventory(&[("suite.yml", &mutated)])
        .expect_err("the precise installation mutation must fail");
    assert!(
        error.contains(diagnostic),
        "wrong mutation diagnostic: {error}"
    );
}

/// Moving a valid installer after the suite cannot satisfy the same-job ordering contract.
#[test]
fn late_installer_is_detected() {
    let baseline = fixture_inventory(&[("suite.yml", INSTALLED)])
        .expect("baseline installer must pass before mutation");
    assert_eq!(baseline.linux_rows, 1);
    let without = replace_once(INSTALLED, INSTALLER, "").expect("one installer must be removed");
    let reordered = format!("{without}{INSTALLER}");
    let error = fixture_inventory(&[("suite.yml", &reordered)])
        .expect_err("moving the installer after the suite must fail");
    assert!(
        error.contains("no unconditional pinned `mold` installer"),
        "{error}"
    );
}

/// An added job in an existing workflow is discovered without extending a job-name whitelist.
#[test]
fn mutation_new_linux_job_is_detected() {
    let baseline =
        fixture_inventory(&[("suite.yml", INSTALLED)]).expect("the original workflow must pass");
    assert_eq!(baseline.suite_jobs, 1);
    let added = include_str!("fixtures/ci_reachability/appended-linux-job.yml");
    let error = fixture_inventory(&[("suite.yml", added)])
        .expect_err("the new Linux suite must require its own installer");
    assert!(
        error.contains("no unconditional pinned `mold` installer"),
        "{error}"
    );
}
