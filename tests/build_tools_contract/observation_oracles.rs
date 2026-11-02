//! Independent expected observations and comparisons for fake processes.
//!
//! This module owns the complete evidence model used by the install cases.

use super::{
    FAKE_MOLD, Invocation, Observation, PrefixState, protocol::comparison_evidence,
    support::Scratch,
};

/// Exact unrelated marker bytes every child observation must preserve.
pub(super) const MARKER: &[u8] = b"preserve";

/// Exact process observations expected from one controlled child run.
pub(super) struct ExpectedObservation {
    /// Required child exit code, including failure-stage statuses.
    status: i32,
    /// Complete ordered fake calls with independently supplied argv and codes.
    calls: Vec<Invocation>,
    /// Normalized checksum stdin, retaining digest and archive basename.
    pub(super) sha_stdin: Vec<u8>,
    /// Exact C translation-unit bytes sent to fake clang.
    pub(super) clang_stdin: Vec<u8>,
    /// Exact child stdout bytes, including direct payload output.
    pub(super) stdout: Vec<u8>,
    /// Diagnostic detail that must appear in raw standard error.
    diagnostic: Option<String>,
    /// Prefix directory, executable bit, and payload bytes after the run.
    prefix: PrefixState,
    /// Marker written only if an injected function unexpectedly ran.
    pub(super) shadow_marker: Vec<u8>,
}

/// Describes prefix existence and exact executable payload state.
pub(super) fn prefix_state(exists: bool, payload: bool) -> PrefixState {
    PrefixState {
        exists,
        mold_executable: payload,
        mold_bytes: payload.then(|| FAKE_MOLD.as_bytes().to_vec()),
    }
}

/// Starts an expectation with four required fields and empty evidence defaults.
pub(super) const fn expected(
    status: i32,
    calls: Vec<Invocation>,
    diagnostic: Option<String>,
    prefix: PrefixState,
) -> ExpectedObservation {
    ExpectedObservation {
        status,
        calls,
        sha_stdin: Vec::new(),
        clang_stdin: Vec::new(),
        stdout: Vec::new(),
        diagnostic,
        prefix,
        shadow_marker: Vec::new(),
    }
}

/// Checks the complete independent process contract without mutating raw data.
pub(super) fn verify_observation(
    scratch: &Scratch,
    observation: &Observation,
    expected: ExpectedObservation,
) -> Result<(), String> {
    let evidence = comparison_evidence(observation, &scratch.path.join("tmp"))?;
    let ExpectedObservation {
        status,
        calls,
        sha_stdin,
        clang_stdin,
        stdout,
        diagnostic,
        prefix,
        shadow_marker,
    } = expected;
    verify_diagnostic(&observation.stderr, diagnostic.as_deref())?;
    let marker = scratch.read_relative("tmp/unrelated.marker")?;
    let wanted = (
        Some(status),
        calls,
        sha_stdin,
        clang_stdin,
        stdout,
        prefix,
        vec!["unrelated.marker".to_owned()],
        MARKER.to_vec(),
        shadow_marker,
    );
    let actual = (
        observation.status.code(),
        evidence.invocations,
        evidence.sha_stdin,
        observation.clang_stdin.clone(),
        observation.stdout.clone(),
        observation.prefix.clone(),
        observation.tmp_children.clone(),
        marker,
        observation.shadow_marker.clone(),
    );
    require(
        actual == wanted,
        "status, calls, stdin, stdout, prefix, temp, and marker evidence",
        &format!("{wanted:?}"),
        &format!("{actual:?}"),
    )
}

/// Checks the expected stderr condition and reports the same observed text.
fn verify_diagnostic(stderr: &[u8], detail: Option<&str>) -> Result<(), String> {
    let observed = String::from_utf8_lossy(stderr);
    detail.map_or_else(
        || require(stderr.is_empty(), "stderr", "empty", observed.as_ref()),
        |expected_detail| {
            require(
                observed.contains(expected_detail),
                "stderr diagnostic",
                expected_detail,
                observed.as_ref(),
            )
        },
    )
}

/// Turns one explicit mismatch into a descriptive, fallible case result.
pub(super) fn require(
    matches: bool,
    oracle: &str,
    wanted: &str,
    observed: &str,
) -> Result<(), String> {
    if matches {
        Ok(())
    } else {
        Err(format!("{oracle}: expected {wanted}, observed {observed}"))
    }
}

/// Compares clean and injected runs without changing their raw observations.
pub(super) fn compare_clean_injected(
    scratch: &Scratch,
    clean: &Observation,
    injected: &Observation,
) -> Result<(), String> {
    let tmp = scratch.path.join("tmp");
    let clean_evidence = comparison_evidence(clean, &tmp)?;
    let injected_evidence = comparison_evidence(injected, &tmp)?;
    require(
        clean_evidence == injected_evidence,
        "normalized calls and checksum",
        &format!("{clean_evidence:?}"),
        &format!("{injected_evidence:?}"),
    )?;
    let raw = |run: &Observation| {
        (
            run.status.code(),
            run.stdout.clone(),
            run.stderr.clone(),
            run.clang_stdin.clone(),
            run.shadow_marker.clone(),
            run.tmp_children.clone(),
            run.prefix.clone(),
        )
    };
    require(
        raw(clean) == raw(injected),
        "raw status, streams, marker, and owned state",
        &format!("{:?}", raw(clean)),
        &format!("{:?}", raw(injected)),
    )
}
