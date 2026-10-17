//! Evaluates the repository Makefile and classifies every reached command.

use std::collections::BTreeSet;
use std::process::Command;

use super::{
    Host, Invocation, Route, Workspace, make_output, make_source, make_syntax, make_tools,
};

/// Reads trace source locations without reconstructing Make prerequisite graphs.
#[path = "ci_reachability_make_trace.rs"]
mod trace;

/// Inputs that remain explicit while one set of Make goals is evaluated.
pub(super) struct MakeRequest<'a> {
    /// Literal goals in their original order.
    pub(super) goals: &'a [String],
    /// Explicit caller selector; ambient MODE is never consulted.
    pub(super) mode: Option<&'a str>,
    /// Reviewed shell assignments attached to this workflow command.
    pub(super) assignments: &'a [String],
    /// Host resolved from the workflow runner row.
    pub(super) host: Host,
}

/// Binds trace rows through the focused source-location reader.
pub(super) fn verify_trace(source: &str, blocks: &[make_output::TraceBlock]) -> Result<(), String> {
    trace::verify_trace(source, blocks)
}

/// Resolves every shell command in a workflow step, including Make prerequisites.
pub(super) fn classify(run: &str, workspace: &Workspace<'_>, host: Host) -> Result<Route, String> {
    make_output::classify_with(run, false, |invocation, controlled| {
        classify_invocation(invocation, controlled, workspace, host)
    })
}

/// Evaluates selected goals with GNU Make after validating source, tools and selectors.
pub(super) fn evaluate_make(
    workspace: &Workspace<'_>,
    request: &MakeRequest<'_>,
) -> Result<make_output::Evaluation, String> {
    let source = workspace
        .directory
        .read_to_string("Makefile")
        .map_err(|error| format!("reading Makefile through the workspace capability: {error}"))?;
    make_source::validate(&source)?;
    make_source::verify_tool_defaults(&source)?;
    let selectors = make_source::required_selectors(&source)?;
    require_explicit_selectors(&selectors, request.mode)?;
    validate_goals(request.goals)?;
    Invocation {
        words: Vec::new(),
        assignments: request.assignments.to_vec(),
        redirects: Vec::new(),
    }
    .validate_environment()?;

    let output = controlled_dry_run(workspace, request)?;
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("GNU Make trace is not UTF-8: {error}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(format!(
            "GNU Make dry run failed with {}; stderr: {}; stdout: {}",
            output.status,
            stderr.trim(),
            stdout.trim()
        ));
    }
    if !stderr.trim().is_empty() {
        return Err(format!(
            "GNU Make wrote unreviewed diagnostics: {}",
            stderr.trim()
        ));
    }
    make_output::classify_traced(stdout, &source, true, |invocation, controlled| {
        make_output::resolve(invocation, controlled, Some(workspace))
    })
}

/// Routes a direct command or starts the controlled evaluator for a literal Make executable.
fn classify_invocation(
    invocation: &Invocation,
    controlled: bool,
    workspace: &Workspace<'_>,
    host: Host,
) -> Result<Route, String> {
    validate_host_flags(invocation, host)?;
    let Some((program, arguments)) = invocation.words.split_first() else {
        return Err("workflow command has no executable".to_owned());
    };
    if matches!(program.as_str(), "make" | "/usr/bin/make") {
        if controlled {
            return Err("workflow cannot invoke the private Make probe".to_owned());
        }
        if !invocation.redirects.is_empty() {
            return Err("workflow Make command has an unreviewed redirection".to_owned());
        }
        let (goals, mode) = make_arguments(arguments)?;
        let request = MakeRequest {
            goals: &goals,
            mode: mode.as_deref(),
            assignments: &invocation.assignments,
            host,
        };
        let evaluation = evaluate_make(workspace, &request)?;
        Ok(evaluation.route)
    } else {
        make_output::resolve(invocation, controlled, Some(workspace))
    }
}

/// Restricts workflow Make arguments to explicit goals and a literal test selector.
fn make_arguments(arguments: &[String]) -> Result<(Vec<String>, Option<String>), String> {
    let mut goals = Vec::new();
    let mut mode = None;
    for argument in arguments {
        if let Some((name, value)) = argument.split_once('=') {
            accept_mode_assignment(name, value, &mut mode)?;
        } else if argument.starts_with('-') || !literal_goal(argument) {
            return Err(format!("unresolved Make option or goal: {argument}"));
        } else {
            goals.push(argument.clone());
        }
    }
    if goals.is_empty() {
        Err("Make invocation has no explicit goal".to_owned())
    } else {
        Ok((goals, mode))
    }
}

/// Records the sole literal MODE assignment, rejecting duplicates before changing state.
/// For example, a second MODE assignment leaves the first value unchanged and fails.
fn accept_mode_assignment(
    name: &str,
    value: &str,
    mode: &mut Option<String>,
) -> Result<(), String> {
    let is_unresolved_assignment = name != "MODE" || mode.is_some() || !literal_selector(value);
    if is_unresolved_assignment {
        return Err(format!("unresolved Make command-line assignment: {name}"));
    }
    *mode = Some(value.to_owned());
    Ok(())
}

/// Ensures all source conditions are selected by the observed host or a literal caller value.
fn require_explicit_selectors(
    selectors: &BTreeSet<String>,
    mode: Option<&str>,
) -> Result<(), String> {
    for selector in selectors {
        match selector.as_str() {
            "BUILD_HOST_OS" => {}
            "MODE" if mode.is_some() => {}
            "MODE" => return Err("Make selector MODE has no explicit workflow value".to_owned()),
            other => return Err(format!("unresolved Make selector: {other}")),
        }
    }
    Ok(())
}

/// Starts only the approved fixed-argv GNU Make dry run with private tool probes.
fn controlled_dry_run(
    workspace: &Workspace<'_>,
    request: &MakeRequest<'_>,
) -> Result<std::process::Output, String> {
    let mut command = Command::new("/usr/bin/make");
    command.args([
        "--dry-run",
        "--trace",
        "--no-print-directory",
        "-B",
        "-j1",
        "--no-builtin-rules",
        "-f",
        "Makefile",
    ]);
    command.args([
        format!("BUILD_HOST_OS={}", request.host.make_value()),
        format!("CARGO={}", make_tools::PROBE_CARGO),
        format!("MDLINT={}", make_tools::PROBE_MDLINT),
        format!("MDTABLEFIX={}", make_tools::PROBE_MDTABLEFIX),
        format!("NIXIE={}", make_tools::PROBE_NIXIE),
        format!("UV={}", make_tools::PROBE_UV),
        format!("WHITAKER={}", make_tools::PROBE_WHITAKER),
    ]);
    if let Some(value) = request.mode {
        command.arg(format!("MODE={value}"));
    }
    command.args(request.goals).current_dir(workspace.path);
    for name in MAKE_ENVIRONMENT_TO_CLEAR {
        command.env_remove(name);
    }
    for assignment in request.assignments {
        let (name, value) = assignment
            .split_once('=')
            .ok_or_else(|| "malformed Make workflow environment assignment".to_owned())?;
        command.env(name, value);
    }
    let output = command
        .output()
        .map_err(|error| format!("starting /usr/bin/make dry run: {error}"))?;
    Ok(output)
}

/// Only ambient Make controls and Makefile inputs which alter command routing are cleared.
const MAKE_ENVIRONMENT_TO_CLEAR: &[&str] = &[
    "BUILD_HOST_OS",
    "BUILD_JOBS",
    "CARGO",
    "CARGO_FLAGS",
    "CLIPPY_FLAGS",
    "CV005_CONTRACTS",
    "CV005_CONTRACTS_REF",
    "GNUMAKEFLAGS",
    "MAKEFLAGS",
    "MAKEFILES",
    "MDLINT",
    "MDTABLEFIX",
    "MDTABLEFIX_RULES",
    "MDTABLEFIX_SELECT",
    "MFLAGS",
    "MODE",
    "NEXTEST_FLAGS",
    "NIXIE",
    "INTEGRATION_TEST_INVENTORY_FILES",
    "INTEGRATION_TEST_INVENTORY_TESTS",
    ".RECIPEPREFIX",
    ".SHELLFLAGS",
    "PYTHONPATH",
    "HYPOTHESIS_STORAGE_DIRECTORY",
    "RUFF_VERSION",
    "RUSTDOC_FLAGS",
    "RUST_FLAGS",
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "SHELL",
    "TARGET",
    "TEST_FLAGS",
    "TYPOS_CONFIG_BUILDER",
    "TYPOS_CONFIG_BUILDER_VERSION",
    "UV",
    "UV_CACHE_DIR",
    "UV_ENV",
    "UV_TOOL_DIR",
    "WHITAKER",
];

/// Refuses empty goals and any selector that would require Make to evaluate shell text.
fn validate_goals(goals: &[String]) -> Result<(), String> {
    if goals.is_empty() || goals.iter().any(|goal| !literal_goal(goal)) {
        Err("Make goals are empty or contain computed syntax".to_owned())
    } else {
        Ok(())
    }
}

/// Rejects values outside the lexical selector language consumed by Make conditions.
fn literal_selector(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

/// Restricts explicit goals to path-free names Make can resolve from this repository.
fn literal_goal(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
        })
}

/// Rejects a mold token on a non-Linux route without qualifying backend behaviour.
fn validate_host_flags(invocation: &Invocation, host: Host) -> Result<(), String> {
    for assignment in &invocation.assignments {
        let Some((name, value)) = assignment.split_once('=') else {
            continue;
        };
        let rejects_non_linux_mold = name == "RUSTFLAGS"
            && host != Host::Linux
            && value
                .split_whitespace()
                .any(|token| token == "-Clink-arg=-fuse-ld=mold");
        if rejects_non_linux_mold {
            return Err("mold compiler flag appears on a non-Linux route".to_owned());
        }
    }
    Ok(())
}

/// Builds trace evidence for a literal target without interpreting its prerequisites.
/// For example, `suite:\n\tcargo test` yields a line-1 trace for `suite`.
#[cfg(test)]
pub(super) fn trace_for_target(
    source: &str,
    target: &str,
    command: &str,
) -> Result<make_output::TraceBlock, String> {
    trace::trace_for_target(source, target, command)
}
