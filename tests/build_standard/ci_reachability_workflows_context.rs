//! Validates literal workflow events and execution context for the bounded inventory.

use super::{Value, Workflow};

/// Validates literal triggers and identifies an event root rather than a callable-only declaration.
pub(super) fn require_trigger(document: &Value) -> Result<bool, String> {
    let root = document.mapping("workflow")?;
    if root
        .keys()
        .any(|key| key.eq_ignore_ascii_case("true") || key.eq_ignore_ascii_case("false"))
    {
        return Err("workflow has an ambiguous boolean trigger key".to_owned());
    }
    match document.required("on", "workflow")? {
        Value::Scalar(event) => require_event_name(event).map(|()| event != "workflow_call"),
        Value::Sequence(events) if !events.is_empty() => require_event_sequence(events),
        other => require_event_mapping(other),
    }
}

/// Validates each event in a sequence instead of accepting an unreadable element by omission.
fn require_event_sequence(events: &[Value]) -> Result<bool, String> {
    let mut event_root = false;
    for event in events {
        let name = event.text("trigger")?;
        require_event_name(name)?;
        event_root |= name != "workflow_call";
    }
    Ok(event_root)
}

/// Requires a literal event identifier; installation inspection covers all events, not just PR events.
fn require_event_name(event: &str) -> Result<(), String> {
    if event == "workflow_run" {
        return Err(
            "workflow_run chain reachability is not resolved by this consumer reader".to_owned(),
        );
    }
    let literal = event.starts_with(|character: char| character.is_ascii_lowercase())
        && event
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '_');
    if literal && !matches!(event, "true" | "false") {
        Ok(())
    } else {
        Err(format!("unresolved workflow trigger event: {event}"))
    }
}

/// Validates a non-empty event mapping without duplicating CV-005's event-reachability policy.
fn require_event_mapping(value: &Value) -> Result<bool, String> {
    let events = value.mapping("workflow trigger")?;
    if events.is_empty() {
        return Err("workflow trigger is empty or indeterminate".to_owned());
    }
    for event in events.keys() {
        require_event_name(event)?;
    }
    Ok(events.keys().any(|event| event != "workflow_call"))
}

/// Reads a non-empty job mapping, preserving malformed or empty documents as measurement gaps.
pub(super) fn jobs(
    workflow: &Workflow,
) -> Result<&std::collections::BTreeMap<String, Value>, String> {
    require_trigger(&workflow.document)?;
    require_execution_context(&workflow.document)?;
    let found = workflow
        .document
        .required("jobs", workflow.path.as_str())?
        .mapping("jobs")?;
    if found.is_empty() {
        return Err(format!("{}: jobs is empty", workflow.path));
    }
    if found.keys().any(String::is_empty) {
        return Err(format!("{}: empty job identifier", workflow.path));
    }
    Ok(found)
}

/// Refuses execution-boundary overrides that the root-relative Make evaluator cannot reproduce.
pub(super) fn require_execution_context(value: &Value) -> Result<(), String> {
    require_run_context(value)?;
    if let Some(defaults) = value.field("defaults")
        && let Some(run) = defaults.mapping("execution defaults")?.get("run")
    {
        run.mapping("default run configuration")?;
        require_run_context(run)?;
    }
    if let Some(environment) = value.field("env") {
        require_environment(environment)?;
    }
    Ok(())
}

/// Accepts the repository root and ordinary POSIX shell declarations, without inferring wrappers.
fn require_run_context(value: &Value) -> Result<(), String> {
    if let Some(directory) = value.optional_text("working-directory", "working-directory")?
        && directory != "."
    {
        return Err(format!(
            "unresolved workflow working-directory: {directory}"
        ));
    }
    if let Some(shell) = value.optional_text("shell", "run shell")?
        && !matches!(shell, "bash" | "sh")
    {
        return Err(format!("unresolved workflow shell: {shell}"));
    }
    Ok(())
}

/// Keeps Make selectors and tool wrappers out of the controlled source evaluator's blind spot.
fn require_environment(environment: &Value) -> Result<(), String> {
    /// Inputs that can change an executable, Make source, selected goal, or command argument route.
    const ROUTING_INPUTS: &[&str] = &[
        "HOME",
        "PATH",
        "SHELL",
        "ENV",
        "BASH_ENV",
        "MAKEFLAGS",
        "MAKEFILES",
        "MFLAGS",
        "CARGO",
        "UV",
        "UV_ENV",
        "WHITAKER",
        "MDLINT",
        "MDTABLEFIX",
        "NIXIE",
        "BUILD_HOST_OS",
        "TEST_FLAGS",
        "NEXTEST_FLAGS",
        "CARGO_FLAGS",
        "BUILD_JOBS",
        "RUST_FLAGS",
        "RUSTDOC_FLAGS",
        "CLIPPY_FLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTDOC",
        "CV005_CONTRACTS_REF",
        "CV005_CONTRACTS",
        "RUFF_VERSION",
        "TYPOS_CONFIG_BUILDER_VERSION",
        "TYPOS_CONFIG_BUILDER",
        "INTEGRATION_TEST_INVENTORY_FILES",
        "INTEGRATION_TEST_INVENTORY_TESTS",
        "PYTHONPATH",
        "PYTHONHOME",
        "PYTHONSTARTUP",
        "PYTEST_ADDOPTS",
        "PYTEST_PLUGINS",
    ];
    for key in environment.mapping("execution environment")?.keys() {
        if key.contains("${{") || ROUTING_INPUTS.contains(&key.to_ascii_uppercase().as_str()) {
            return Err(format!(
                "unresolved workflow execution environment input: {key}"
            ));
        }
    }
    Ok(())
}
