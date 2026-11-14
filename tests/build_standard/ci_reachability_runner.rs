//! Resolves literal runner labels and finite matrix rows to supported host operating systems.

use super::{Host, Value, matrix};

/// Resolves every finite runner row and fails when any row has no provable operating system.
pub(super) fn hosts(job: &Value) -> Result<Vec<Host>, String> {
    let runner = job.required("runs-on", "job")?;
    let rows = matrix::rows(job.field("strategy"))?;
    let mut hosts = Vec::new();
    for row in rows {
        let resolved = matrix::substitute(runner, &row)?;
        hosts.push(resolve(&resolved)?);
    }
    if hosts.is_empty() {
        Err("runner matrix has no resolved rows".to_owned())
    } else {
        Ok(hosts)
    }
}

/// Selects runner-label evidence from the supported scalar, list, or group mapping shapes.
fn resolve(runner: &Value) -> Result<Host, String> {
    match runner {
        Value::Scalar(_) => host_from_values(std::slice::from_ref(runner)),
        Value::Sequence(labels) => host_from_values(labels),
        Value::Mapping(fields) => host_from_group(fields),
        Value::Empty => Err("runs-on is empty".to_owned()),
    }
}

/// A runner label that has passed the literal-label grammar.
///
/// Workflow runner rows are untrusted text, so the host classifier accepts only
/// values that this parser has already proved non-empty and free of unresolved
/// expressions. For example, `parse` refuses `" "` and `${{ runner.os }}`
/// while accepting `ubuntu-latest`.
struct RunnerLabel<'a>(&'a str);

impl<'a> RunnerLabel<'a> {
    /// Reads one literal label from a workflow scalar.
    fn parse(value: &'a Value, context: &str) -> Result<Self, String> {
        let label = value.text(context)?;
        if label.trim().is_empty() || contains_runner_expression_marker(label) {
            Err(format!("{context} is empty or computed"))
        } else {
            Ok(Self(label))
        }
    }

    /// Resolves the label to the operating system it names, if any.
    fn host(&self) -> Option<Host> { host_label(self.0) }
}

/// Resolves a list of required labels as one runner rather than alternative runners.
fn host_from_values(values: &[Value]) -> Result<Host, String> {
    if values.is_empty() {
        return Err("runs-on label list is empty".to_owned());
    }
    let labels = values
        .iter()
        .map(|value| RunnerLabel::parse(value, "runs-on label"))
        .collect::<Result<Vec<_>, _>>()?;
    host_from_labels(&labels)
}

/// Requires labels and validates any optional opaque group name without using it as OS proof.
fn host_from_group(fields: &std::collections::BTreeMap<String, Value>) -> Result<Host, String> {
    if fields
        .keys()
        .any(|key| !matches!(key.as_str(), "group" | "labels"))
    {
        return Err("runs-on mapping may contain only `group` and `labels`".to_owned());
    }
    if let Some(group) = fields.get("group") {
        RunnerLabel::parse(group, "runs-on group")?;
    }
    let labels_value = fields
        .get("labels")
        .ok_or_else(|| "runs-on mapping is missing `labels`".to_owned())?;
    match labels_value {
        Value::Scalar(_) => host_from_values(std::slice::from_ref(labels_value)),
        Value::Sequence(values) => host_from_values(values),
        _ => Err("runs-on `labels` must be a scalar or sequence".to_owned()),
    }
}

/// Detects either delimiter used by an unresolved runner-label expression.
///
/// For example, `ubuntu-latest` has no marker, while `${{ runner.os }}` does.
fn contains_runner_expression_marker(label: &str) -> bool {
    label.contains("${{") || label.contains("}}")
}

/// Combines known OS labels and refuses conflicting or absent operating-system evidence.
fn host_from_labels(labels: &[RunnerLabel<'_>]) -> Result<Host, String> {
    let mut found = None;
    for label in labels {
        if let Some(host) = label.host() {
            if found.is_some_and(|existing| existing != host) {
                return Err("runner labels identify conflicting operating systems".to_owned());
            }
            found = Some(host);
        }
    }
    found.ok_or_else(|| "runner labels do not prove a supported operating system".to_owned())
}

/// Maps only recognized operating-system labels; architecture labels carry no OS evidence.
fn host_label(label: &str) -> Option<Host> {
    let normalized = label.to_ascii_lowercase();
    if is_supported_normalized_linux_label(&normalized) {
        Some(Host::Linux)
    } else if normalized == "windows" || normalized.starts_with("windows-") {
        Some(Host::Windows)
    } else if normalized == "macos" || normalized.starts_with("macos-") {
        Some(Host::Darwin)
    } else {
        None
    }
}

/// Recognizes a normalized Linux runner name and the supported Linux label prefixes.
///
/// For example, `ubuntu-latest` and `linux-x86_64` are Linux labels, while
/// `debian-latest` is not classified here.
fn is_supported_normalized_linux_label(label: &str) -> bool {
    label == "ubicloud-standard-2"
        || label == "linux"
        || label.starts_with("linux-")
        || label.starts_with("ubuntu-")
}
