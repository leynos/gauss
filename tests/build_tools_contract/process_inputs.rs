//! Validates explicit environment fixtures for controlled child processes.
//!
//! Only typed case inputs are applied; production child launch stays in `process.rs`.

use std::process::Command;

use super::{EnvironmentChange, support::Scratch};

/// Applies only the case-owned environment changes to one child command.
pub(super) fn apply_environment_changes(
    command: &mut Command,
    scratch: &Scratch,
    changes: &[EnvironmentChange],
) -> Result<(), String> {
    let mut seen = Vec::new();
    for change in changes {
        let name = change.name.as_str();
        if seen.contains(&name) {
            return Err(format!("duplicate child environment fixture: {name}"));
        }
        if !allowed_input(name) {
            return Err(format!(
                "child environment fixture is not an allowed input: {name}"
            ));
        }
        if let Some(value) = &change.value {
            validate_input_value(scratch, name, value)?;
            command.env(name, value);
        } else {
            command.env_remove(name);
        }
        seen.push(name);
    }
    Ok(())
}

/// Recognizes only environment inputs exposed by the controlled cases.
fn allowed_input(name: &str) -> bool {
    matches!(
        name,
        "MOLD_VERSION_FILE"
            | "MOLD_SHA256SUMS_FILE"
            | "RUST_TOOLCHAIN_FILE"
            | "BUILD_TOOLS_PREFIX"
            | "BUILD_TOOLS_TARGET"
            | "CARGO_BUILD_TARGET"
            | "CARGO_ENCODED_RUSTFLAGS"
            | "BUILD_TOOLS_MOLD_REQUIRED"
    )
}

/// Rejects path fixtures that do not remain below owned scratch.
fn validate_input_value(scratch: &Scratch, name: &str, value: &str) -> Result<(), String> {
    let is_path = matches!(
        name,
        "MOLD_VERSION_FILE" | "MOLD_SHA256SUMS_FILE" | "RUST_TOOLCHAIN_FILE" | "BUILD_TOOLS_PREFIX"
    );
    if !is_path || value.is_empty() {
        return Ok(());
    }
    let root = scratch.path.as_str();
    let after_root = value
        .strip_prefix(root)
        .ok_or_else(|| format!("path fixture escapes owned scratch: {name}"))?;
    if after_root.is_empty() {
        return Ok(());
    }
    let below_root = after_root
        .strip_prefix('/')
        .ok_or_else(|| format!("path fixture escapes owned scratch: {name}"))?;
    let relative_path = below_root.strip_suffix('/').unwrap_or(below_root);
    if relative_path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("path fixture escapes owned scratch: {name}"));
    }
    Ok(())
}
