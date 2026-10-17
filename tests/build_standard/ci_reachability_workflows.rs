//! Inventories local workflows and requires same-job installation before resolved Linux suites.

use camino::Utf8PathBuf;
use std::collections::BTreeSet;

use super::{
    ACTION_REVISION, Coverage, Host, Route, Value, Workflow, Workspace, make_routes, parser, runner,
};

/// Groups trigger and execution-context checks for this workflow reader.
#[path = "ci_reachability_workflows_context.rs"]
mod context;

use context::{jobs, require_execution_context, require_trigger};

/// Reads every YAML workflow through the capability root; a missing inventory is not evidence.
pub(super) fn read_inventory(workspace: &Workspace<'_>) -> Result<Vec<Workflow>, String> {
    let entries = workspace
        .directory
        .read_dir(".github/workflows")
        .map_err(|error| format!("reading workflow inventory: {error}"))?;
    let mut inventory = Vec::new();
    for entry_result in entries {
        let entry = entry_result.map_err(|error| format!("reading workflow entry: {error}"))?;
        let name = entry
            .file_name()
            .map_err(|error| format!("reading workflow entry name: {error}"))?;
        let has_yaml_extension = name
            .rsplit_once('.')
            .is_some_and(|(_, extension)| matches!(extension, "yml" | "yaml"));
        if !has_yaml_extension {
            continue;
        }
        if !entry
            .file_type()
            .map_err(|error| format!("workflow entry type: {error}"))?
            .is_file()
        {
            return Err(format!("workflow {name} is not a regular file"));
        }
        let path = Utf8PathBuf::from(".github/workflows").join(name);
        let source = workspace
            .directory
            .read_to_string(&path)
            .map_err(|error| format!("reading {path}: {error}"))?;
        let document = parser::parse(&source).map_err(|error| format!("{path}: {error}"))?;
        inventory.push(Workflow { path, document });
    }
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    if inventory.is_empty() {
        return Err("workflow inventory is empty".to_owned());
    }
    Ok(inventory)
}

/// Recognizes only the reviewed remote reusable workflow that never checks out or executes PR code.
fn approved_non_suite_call(reference: &str) -> bool {
    reference
        == format!(
            "leynos/shared-actions/.github/workflows/dependabot-automerge.yml@{ACTION_REVISION}"
        )
}

/// Resolves a supported local reusable call to an exact inventory entry.
fn local_callee<'a>(reference: &str, inventory: &'a [Workflow]) -> Result<&'a Workflow, String> {
    let path = reference
        .strip_prefix("./.github/workflows/")
        .ok_or_else(|| format!("unprovable reusable workflow: {reference}"))?;
    if path.contains('/') || contains_unreviewed_reference_marker(path) {
        return Err(format!("unsupported local reusable workflow: {reference}"));
    }
    let expected = Utf8PathBuf::from(".github/workflows").join(path);
    inventory
        .iter()
        .find(|workflow| workflow.path == expected)
        .ok_or_else(|| format!("missing reusable workflow: {reference}"))
}

/// Detects remote or computed syntax that cannot name an inventory file literally.
///
/// For example, `ci.yml` has no such marker, while `ci.yml@main` does.
fn contains_unreviewed_reference_marker(path: &str) -> bool {
    path.contains('@') || path.contains("${{")
}

/// Requires a callee to declare the supported `workflow_call` trigger, not
/// merely exist on disk.
fn require_reusable_trigger(workflow: &Workflow) -> Result<(), String> {
    let trigger = workflow.document.required("on", "reusable workflow")?;
    let callable = match trigger {
        Value::Scalar(event) => event == "workflow_call",
        Value::Sequence(events) => events
            .iter()
            .any(|event| matches!(event, Value::Scalar(name) if name == "workflow_call")),
        other => other
            .mapping("reusable workflow trigger")?
            .contains_key("workflow_call"),
    };
    if callable {
        Ok(())
    } else {
        Err(format!("{} does not declare workflow_call", workflow.path))
    }
}

/// Follows local reusable edges before inspecting steps; cycles and missing targets fail closed.
fn require_calls(
    workflow: &Workflow,
    inventory: &[Workflow],
    ancestors: &mut Vec<Utf8PathBuf>,
) -> Result<BTreeSet<Utf8PathBuf>, String> {
    if ancestors.contains(&workflow.path) {
        return Err(format!("reusable workflow cycle at {}", workflow.path));
    }
    ancestors.push(workflow.path.clone());
    let mut reachable = BTreeSet::from([workflow.path.clone()]);
    for job in jobs(workflow)?.values() {
        reachable.extend(require_job_call(job, inventory, ancestors)?);
    }
    let removed = ancestors
        .pop()
        .ok_or_else(|| "reusable traversal lost its current path".to_owned())?;
    if removed != workflow.path {
        return Err("reusable traversal removed a different path".to_owned());
    }
    Ok(reachable)
}

/// Resolves one reusable job without transferring a caller's installation to the callee.
fn require_job_call(
    job: &Value,
    inventory: &[Workflow],
    ancestors: &mut Vec<Utf8PathBuf>,
) -> Result<BTreeSet<Utf8PathBuf>, String> {
    job.mapping("job")?;
    let Some(reference) = job.optional_text("uses", "job uses")? else {
        return Ok(BTreeSet::new());
    };
    if job.field("steps").is_some() || job.field("runs-on").is_some() {
        return Err("reusable job mixes uses with steps or runs-on".to_owned());
    }
    if !approved_non_suite_call(reference) {
        let callee = local_callee(reference, inventory)?;
        require_reusable_trigger(callee)?;
        return require_calls(callee, inventory, ancestors);
    }
    Ok(BTreeSet::new())
}

/// Classifies only exact reviewed action references; unknown actions cannot hide a test suite.
fn action_route(reference: &str) -> Result<Route, String> {
    if reference
        == format!("leynos/shared-actions/.github/actions/generate-coverage@{ACTION_REVISION}")
    {
        return Ok(Route::Suite);
    }
    let shared = [
        "setup-rust",
        "install-mdtablefix",
        "install-whitaker",
        "upload-codescene-coverage",
    ];
    let pinned_shared = shared.iter().any(|name| {
        reference == format!("leynos/shared-actions/.github/actions/{name}@{ACTION_REVISION}")
    });
    let external = [
        "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
        "DavidAnson/markdownlint-cli2-action@21c1be1b93ad9ed58fa840aacc3f279cde2a72ff",
    ];
    if pinned_shared || external.contains(&reference) {
        Ok(Route::NonSuite)
    } else {
        Err(format!("unprovable action reference: {reference}"))
    }
}

/// Rejects soft or conditional execution where the installation or suite must bind its job.
fn require_binding(step: &Value, context: &str) -> Result<(), String> {
    if step.field("if").is_some() || step.field("continue-on-error").is_some() {
        return Err(format!(
            "{context} must be unconditional and must propagate failure"
        ));
    }
    Ok(())
}

/// Identifies the current `install-mold` input, without claiming the later complete-tool preflight.
fn installs_mold(step: &Value) -> Result<bool, String> {
    let expected = format!("leynos/shared-actions/.github/actions/setup-rust@{ACTION_REVISION}");
    if step.optional_text("uses", "step uses")? != Some(expected.as_str()) {
        return Ok(false);
    }
    let enabled = step
        .field("with")
        .map(|inputs| inputs.optional_text("install-mold", "install-mold"))
        .transpose()?
        .flatten()
        == Some("true");
    if enabled {
        require_binding(step, "`mold` installer")?;
    }
    Ok(enabled)
}

/// Classifies a step's sole executable route, keeping action and shell forms distinct.
fn step_route(step: &Value, workspace: &Workspace<'_>, host: Host) -> Result<Route, String> {
    step.mapping("step")?;
    require_execution_context(step)?;
    match (
        step.optional_text("uses", "step uses")?,
        step.optional_text("run", "step run")?,
    ) {
        (Some(reference), None) => action_route(reference),
        (None, Some(run)) => make_routes::classify(run, workspace, host),
        _ => Err("step must have exactly one supported uses or run route".to_owned()),
    }
}

/// Installation state belongs to one resolved runner row, never another job.
#[derive(Clone, Copy)]
struct Provisioning {
    /// The statically resolved operating system; this does not prove architecture.
    host: Host,
    /// Whether an unconditional current installer has already completed in source order.
    installed: bool,
}

/// Checks the suite's binding and failure boundary against the row's current installation state.
fn require_suite_binding(step: &Value, job: &Value, state: Provisioning) -> Result<(), String> {
    require_binding(step, "suite step")?;
    if state.host == Host::Linux && !state.installed {
        return Err(
            "Linux suite has no unconditional pinned `mold` installer before it".to_owned(),
        );
    }
    if job.field("continue-on-error").is_some() {
        return Err("suite job must propagate failure".to_owned());
    }
    Ok(())
}

/// Checks one runner row in source order; an installer in another job or after a suite never counts.
fn inspect_row(job: &Value, workspace: &Workspace<'_>, host: Host) -> Result<Route, String> {
    let steps = job.required("steps", "job")?.sequence("steps")?;
    if steps.is_empty() {
        return Err("job steps is empty".to_owned());
    }
    let mut state = Provisioning {
        host,
        installed: false,
    };
    let mut route = Route::NonSuite;
    for step in steps {
        state.installed |= installs_mold(step)?;
        let observed = step_route(step, workspace, host)?;
        if observed == Route::Suite {
            require_suite_binding(step, job, state)?;
        }
        route = route.combine(observed);
    }
    Ok(route)
}

/// Measures each resolved row of an ordinary job; reusable declarations are checked via their edges.
fn inspect_job(job: &Value, context: &str, workspace: &Workspace<'_>) -> Result<Coverage, String> {
    require_execution_context(job)?;
    if job.field("container").is_some() || job.field("services").is_some() {
        return Err(format!(
            "{context}: uninspected job container or service execution route"
        ));
    }
    if job.field("uses").is_some() {
        return Ok(Coverage {
            suite_jobs: 0,
            linux_rows: 0,
        });
    }
    let hosts = runner::hosts(job).map_err(|error| format!("{context}: {error}"))?;
    let mut suite = Route::NonSuite;
    let mut linux_rows = 0;
    for host in hosts {
        let row = inspect_row(job, workspace, host)
            .map_err(|error| format!("{context}:{host:?}: {error}"))?;
        suite = suite.combine(row);
        if host == Host::Linux && row == Route::Suite {
            linux_rows += 1;
        }
    }
    Ok(Coverage {
        suite_jobs: usize::from(suite == Route::Suite),
        linux_rows,
    })
}

/// Inspects all declarations as well as reusable edges, so a new YAML file is never outside scope.
pub(super) fn inspect_inventory(
    inventory: &[Workflow],
    workspace: &Workspace<'_>,
) -> Result<Coverage, String> {
    if inventory.is_empty() {
        return Err("workflow inventory is empty".to_owned());
    }
    let reachable = event_reachable_workflows(inventory)?;
    let mut coverage = Coverage {
        suite_jobs: 0,
        linux_rows: 0,
    };
    for workflow in inventory {
        let measured = inspect_workflow(workflow, workspace, reachable.contains(&workflow.path))?;
        coverage.suite_jobs += measured.suite_jobs;
        coverage.linux_rows += measured.linux_rows;
    }
    if coverage.suite_jobs == 0 {
        return Err(
            "workflow inventory contains no resolved suite jobs reachable from an event".to_owned(),
        );
    }
    Ok(coverage)
}

/// Resolves every reusable call before measuring jobs, retaining trigger-validation order.
/// For example, a non-triggering document still has all its calls checked for gaps.
fn event_reachable_workflows(inventory: &[Workflow]) -> Result<BTreeSet<Utf8PathBuf>, String> {
    let mut reachable = BTreeSet::new();
    for workflow in inventory {
        let visited = require_calls(workflow, inventory, &mut Vec::new())?;
        if require_trigger(&workflow.document)? {
            reachable.extend(visited);
        }
    }
    Ok(reachable)
}

/// Inspects all declared jobs but counts coverage only for an event-reachable workflow.
/// For example, an unreachable malformed job still fails before any later workflow is read.
fn inspect_workflow(
    workflow: &Workflow,
    workspace: &Workspace<'_>,
    is_reachable: bool,
) -> Result<Coverage, String> {
    let mut coverage = Coverage {
        suite_jobs: 0,
        linux_rows: 0,
    };
    for (job_id, job) in jobs(workflow)? {
        let context = format!("{}:{job_id}", workflow.path);
        let measured = inspect_job(job, &context, workspace)?;
        if is_reachable {
            coverage.suite_jobs += measured.suite_jobs;
            coverage.linux_rows += measured.linux_rows;
        }
    }
    Ok(coverage)
}
