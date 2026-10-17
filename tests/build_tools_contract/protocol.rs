//! Isolation records, strict fake-call parsing, and comparison evidence.
//!
//! Raw process records remain available to callers; the comparison view only
//! rewrites one validated, owned download-directory root.

use camino::{Utf8Path, Utf8PathBuf};

use super::{
    Invocation, Observation, PrefixState,
    model::{ComparisonEvidence, RawOutput, Recordings},
};

/// Builds comparison evidence without changing the original observation.
pub(super) fn comparison_evidence(
    observation: &Observation,
    tmpdir: &Utf8Path,
) -> Result<ComparisonEvidence, String> {
    let (invocations, sha_stdin) =
        normalize_comparison_inputs(&observation.invocations, &observation.sha_stdin, tmpdir)?;
    Ok(ComparisonEvidence {
        invocations,
        sha_stdin,
    })
}

fn normalize_comparison_inputs(
    invocations: &[Invocation],
    sha_stdin: &[u8],
    tmpdir: &Utf8Path,
) -> Result<(Vec<Invocation>, Vec<u8>), String> {
    if !tmpdir.is_absolute() || tmpdir.as_str().ends_with('/') {
        return Err("comparison TMPDIR is not an exact absolute directory".to_owned());
    }
    let sha_text = std::str::from_utf8(sha_stdin)
        .map_err(|error| format!("sha256sum stdin is not UTF-8: {error}"))?;
    let root = discover_download_root(invocations, sha_text, tmpdir)?;

    let compared = normalize_invocations(invocations, root.as_deref());
    let compared_sha = normalized_text(sha_text, root.as_deref()).into_bytes();
    Ok((compared, compared_sha))
}

/// Finds the one validated download root shared by calls and checksum input.
fn discover_download_root(
    invocations: &[Invocation],
    sha_text: &str,
    tmpdir: &Utf8Path,
) -> Result<Option<String>, String> {
    let mut root = None;
    for invocation in invocations {
        for argument in &invocation.argv {
            collect_download_root(argument, tmpdir, &mut root)?;
        }
    }
    collect_download_root(sha_text, tmpdir, &mut root)?;
    Ok(root)
}

/// Builds normalized call records while preserving the caller's raw records.
fn normalize_invocations(invocations: &[Invocation], root: Option<&str>) -> Vec<Invocation> {
    let mut compared = invocations.to_vec();
    for invocation in &mut compared {
        for argument in &mut invocation.argv {
            *argument = normalized_text(argument, root);
        }
    }
    compared
}

fn collect_download_root(
    text: &str,
    tmpdir: &Utf8Path,
    known_root: &mut Option<String>,
) -> Result<(), String> {
    let marker = tmpdir.as_str();
    let mut remaining = text;
    while let Some((before, after)) = remaining.split_once(marker) {
        if before
            .chars()
            .last()
            .is_some_and(|character| !character.is_ascii_whitespace())
        {
            return Err("scratch path occurs outside a path boundary".to_owned());
        }
        record_owned_download_root(after, marker, known_root)?;
        remaining = after;
    }
    Ok(())
}

fn record_owned_download_root(
    after: &str,
    marker: &str,
    known_root: &mut Option<String>,
) -> Result<(), String> {
    let suffix = after
        .strip_prefix("/gauss-build-tools.")
        .ok_or_else(|| "unexpected path under controlled TMPDIR".to_owned())?;
    let random_end = suffix
        .find(|character: char| !character.is_ascii_alphanumeric())
        .unwrap_or(suffix.len());
    let random = suffix
        .get(..random_end)
        .ok_or_else(|| "download directory has an invalid string boundary".to_owned())?;
    let remainder = suffix
        .get(random_end..)
        .ok_or_else(|| "download directory has an invalid string boundary".to_owned())?;
    if random.is_empty() || !download_tail_has_boundary(remainder) {
        return Err("download directory is not an exact path component".to_owned());
    }
    let path_tail = remainder
        .split_ascii_whitespace()
        .next()
        .unwrap_or_default();
    if path_tail.split('/').any(|component| component == "..") {
        return Err("owned download path contains traversal".to_owned());
    }
    let root = format!("{marker}/gauss-build-tools.{random}");
    if known_root.as_ref().is_some_and(|known| known != &root) {
        return Err("fake calls refer to multiple download directories".to_owned());
    }
    *known_root = Some(root);
    Ok(())
}

/// Reports whether the download-root suffix ends at an ASCII path boundary.
fn download_tail_has_boundary(remainder: &str) -> bool {
    match remainder.chars().next() {
        None | Some('/') => true,
        Some(character) => character.is_ascii_whitespace(),
    }
}

fn normalized_text(text: &str, root: Option<&str>) -> String {
    root.map_or_else(
        || text.to_owned(),
        |download_root| text.replace(download_root, "<OWNED_DOWNLOAD_DIR>"),
    )
}

/// Parses process output after validating exact call and status records.
pub(super) fn parse_observation(
    raw: RawOutput,
    recordings: Recordings,
) -> Result<Observation, String> {
    Ok(Observation {
        status: raw.status,
        stdout: raw.stdout,
        stderr: raw.stderr,
        invocations: parse_log(&recordings.call_log)?,
        sha_stdin: recordings.sha_stdin,
        clang_stdin: recordings.clang_stdin,
        tmp_children: Vec::new(),
        prefix: PrefixState {
            exists: false,
            mold_executable: false,
            mold_bytes: None,
        },
        shadow_marker: Vec::new(),
    })
}

fn parse_log(bytes: &[u8]) -> Result<Vec<Invocation>, String> {
    let log = std::str::from_utf8(bytes)
        .map_err(|error| format!("fake call log is not UTF-8: {error}"))?;
    if log.is_empty() {
        return Ok(Vec::new());
    }
    if !log.ends_with('\n') {
        return Err("fake call log ends with an incomplete record".to_owned());
    }
    let mut rows = log.split_terminator('\n');
    let mut invocations = Vec::new();
    while let Some(call) = rows.next() {
        let (tool, argv) = parse_call_record(call)?;
        let status_record = rows
            .next()
            .ok_or_else(|| format!("fake call {tool} has no status record"))?;
        let status = parse_status_record(status_record, &tool)?;
        invocations.push(Invocation { tool, argv, status });
    }
    Ok(invocations)
}

fn parse_call_record(call: &str) -> Result<(String, Vec<String>), String> {
    let mut fields = call.split('\t');
    let kind = fields.next();
    let tool_field = fields.next();
    let count_field = fields.next();
    let (Some("CALL"), Some(tool_name), Some(count_text)) = (kind, tool_field, count_field) else {
        return Err(format!("invalid fake call record: {call}"));
    };
    if !known_tool(tool_name) {
        return Err(format!("unknown fake call tool: {tool_name}"));
    }
    let argument_count = count_text
        .parse::<usize>()
        .map_err(|error| format!("invalid argv count in fake call record: {error}"))?;
    let argv: Vec<String> = fields.map(str::to_owned).collect();
    if argv.len() != argument_count {
        return Err(format!("fake call record has incomplete argv: {call}"));
    }
    Ok((tool_name.to_owned(), argv))
}

fn parse_status_record(record: &str, expected_tool: &str) -> Result<i32, String> {
    let mut fields = record.split('\t');
    let kind = fields.next();
    let tool = fields.next();
    let code_field = fields.next();
    if (kind, tool) != (Some("STATUS"), Some(expected_tool)) || fields.next().is_some() {
        return Err(format!("invalid fake status record: {record}"));
    }
    let code_text = code_field.ok_or_else(|| format!("invalid fake status record: {record}"))?;
    let status = code_text
        .parse::<i32>()
        .map_err(|error| format!("invalid fake process status: {error}"))?;
    if !(0..=255).contains(&status) {
        return Err(format!("fake process status is outside 0..=255: {status}"));
    }
    Ok(status)
}

fn known_tool(tool: &str) -> bool {
    matches!(
        tool,
        "dirname"
            | "uname"
            | "curl"
            | "sha256sum"
            | "tar"
            | "rustup"
            | "mold"
            | "prefix-mold"
            | "clang"
            | "cargo"
            | "rustc"
            | "cc"
            | "gcc"
            | "c++"
            | "clang++"
            | "make"
            | "cmake"
            | "ninja"
            | "git"
            | "wget"
            | "whitaker-installer"
    )
}

/// Validates path syntax before a capability-relative scratch operation.
pub(super) fn checked_relative(path: &str) -> Result<Utf8PathBuf, String> {
    if invalid_relative_root(path) || invalid_relative_components(path) {
        return Err(format!("scratch path is not a safe relative path: {path}"));
    }
    Ok(Utf8PathBuf::from(path))
}

/// Identifies empty paths and paths rooted at the filesystem separator.
fn invalid_relative_root(path: &str) -> bool {
    path.is_empty() || path.starts_with('/')
}

/// Identifies separators and components forbidden in a relative scratch path.
fn invalid_relative_components(path: &str) -> bool {
    path.contains('\\') || path.split('/').any(|part| matches!(part, "" | "." | ".."))
}

#[cfg(test)]
mod tests {
    //! Pure log and comparison helpers preserve argv order and raw evidence.

    use camino::Utf8Path;

    use super::super::Invocation;
    use super::{normalize_comparison_inputs, parse_log};

    #[test]
    fn log_records_preserve_exact_argv_and_status() {
        assert_eq!(parse_log(b""), Ok(Vec::new()));
        assert_eq!(
            parse_log(b"CALL\tcurl\t2\t--output\t/archive\nSTATUS\tcurl\t0\n"),
            Ok(vec![Invocation {
                tool: "curl".to_owned(),
                argv: vec!["--output".to_owned(), "/archive".to_owned()],
                status: 0,
            }])
        );
    }

    #[test]
    fn comparison_normalizes_only_the_correlated_owned_root() {
        let tmpdir = Utf8Path::new("/tmp/owned");
        let root = "/tmp/owned/gauss-build-tools.A1b2c3";
        let digest = "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d";
        let calls = vec![Invocation {
            tool: "curl".to_owned(),
            argv: vec![
                "--output".to_owned(),
                format!("{root}/mold-2.41.0-x86_64-linux.tar.gz"),
            ],
            status: 0,
        }];
        let sha_stdin = format!("{digest}  {root}/mold-2.41.0-x86_64-linux.tar.gz\n").into_bytes();
        let raw_calls = calls.clone();
        let raw_sha = sha_stdin.clone();
        let compared = normalize_comparison_inputs(&calls, &sha_stdin, tmpdir);
        let expected_calls = vec![Invocation {
            tool: "curl".to_owned(),
            argv: vec![
                "--output".to_owned(),
                "<OWNED_DOWNLOAD_DIR>/mold-2.41.0-x86_64-linux.tar.gz".to_owned(),
            ],
            status: 0,
        }];
        let expected_sha =
            format!("{digest}  <OWNED_DOWNLOAD_DIR>/mold-2.41.0-x86_64-linux.tar.gz\n")
                .into_bytes();
        assert_eq!(compared, Ok((expected_calls, expected_sha)));
        assert_eq!(calls, raw_calls);
        assert_eq!(sha_stdin, raw_sha);
    }

    #[test]
    fn comparison_rejects_owned_root_traversal() {
        let calls = vec![Invocation {
            tool: "curl".to_owned(),
            argv: vec!["/tmp/owned/gauss-build-tools.A1b2c3/../etc".to_owned()],
            status: 0,
        }];
        assert!(normalize_comparison_inputs(&calls, b"", Utf8Path::new("/tmp/owned")).is_err());
        let unrelated = vec![Invocation {
            tool: "curl".to_owned(),
            argv: vec!["unrelated-/tmp/owned/gauss-build-tools.A1b2c3".to_owned()],
            status: 0,
        }];
        assert!(normalize_comparison_inputs(&unrelated, b"", Utf8Path::new("/tmp/owned")).is_err());
    }
}
