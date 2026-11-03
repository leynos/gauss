//! Fake `rustup` response configuration for the build-tool contracts.
//!
//! The fake dispatches on a per-subcommand call counter, so every ordinal in a
//! sequence needs its own status and output pair. Keeping that mapping here
//! leaves the process module focused on launching children.

use std::process::Command;

use super::{Failure, RustupFixture};

/// One fake `rustup` invocation's configured response.
///
/// The ordinal selects which response a call sees. Install calls record no
/// output.
#[derive(Clone, Copy)]
struct RustupCall<'a> {
    /// One-based position within the subcommand's call sequence.
    ordinal: usize,
    /// Exit status the fake reports for this call.
    status: i32,
    /// Standard output the fake prints, when the subcommand has any.
    output: Option<&'a str>,
}

impl<'a> RustupCall<'a> {
    /// Builds one configured response for a call ordinal.
    const fn new(ordinal: usize, status: i32, output: Option<&'a str>) -> Self {
        Self {
            ordinal,
            status,
            output,
        }
    }
}

/// Exit statuses the fake reports for each subcommand's three calls.
struct RustupStatuses {
    /// Declared-channel, stable then nightly install statuses.
    install: [i32; 3],
    /// Declared-channel, stable then nightly toolchain-list statuses.
    list: [i32; 3],
    /// Declared-channel, stable then nightly component-list statuses.
    component: [i32; 3],
}

impl RustupStatuses {
    /// Reads the fixture statuses and applies the configured failure stage.
    const fn new(fixture: &RustupFixture, failure: Option<Failure>) -> Self {
        let mut statuses = Self {
            install: fixture.install_status,
            list: fixture.list_status,
            component: fixture.component_status,
        };
        statuses.apply_failure(failure);
        statuses
    }

    /// Overrides the status of the call the failure stage targets.
    const fn apply_failure(&mut self, failure: Option<Failure>) {
        let [declared_install, stable_install, nightly_install] = &mut self.install;
        let [declared_list, stable_list, nightly_list] = &mut self.list;
        let [declared_components, stable_components, nightly_components] = &mut self.component;
        match failure {
            Some(Failure::DeclaredInstall) => *declared_install = 97,
            Some(Failure::StableInstall) => *stable_install = 97,
            Some(Failure::NightlyInstall) => *nightly_install = 97,
            Some(Failure::DeclaredToolchainList) => *declared_list = 97,
            Some(Failure::StableToolchainList) => *stable_list = 97,
            Some(Failure::NightlyToolchainList) => *nightly_list = 97,
            Some(Failure::DeclaredComponentList) => *declared_components = 97,
            Some(Failure::StableComponentList) => *stable_components = 97,
            Some(Failure::NightlyComponentList) => *nightly_components = 97,
            _ => {}
        }
    }
}

/// Writes every configured response for one fake subcommand.
pub(super) fn configure_rustup(
    command: &mut Command,
    fixture: &RustupFixture,
    failure: Option<Failure>,
) -> Result<(), String> {
    let statuses = RustupStatuses::new(fixture, failure);
    write_rustup_calls(command, "INSTALL", &install_calls(statuses.install))?;
    write_rustup_calls(
        command,
        "LIST",
        &list_calls(statuses.list, &fixture.list_output),
    )?;
    write_rustup_calls(
        command,
        "COMPONENT",
        &component_calls(statuses.component, &fixture.component_output),
    )?;
    Ok(())
}

/// Builds the three install-call responses, which print nothing.
const fn install_calls(statuses: [i32; 3]) -> [RustupCall<'static>; 3] {
    let [declared, stable, nightly] = statuses;
    [
        RustupCall::new(1, declared, None),
        RustupCall::new(2, stable, None),
        RustupCall::new(3, nightly, None),
    ]
}

/// Builds the three toolchain-list responses with their inventories.
const fn list_calls(statuses: [i32; 3], outputs: &[String; 3]) -> [RustupCall<'_>; 3] {
    let [declared, stable, nightly] = statuses;
    let [declared_output, stable_output, nightly_output] = outputs;
    [
        RustupCall::new(1, declared, Some(declared_output.as_str())),
        RustupCall::new(2, stable, Some(stable_output.as_str())),
        RustupCall::new(3, nightly, Some(nightly_output.as_str())),
    ]
}

/// Builds the three component-list responses with their inventories.
const fn component_calls(statuses: [i32; 3], outputs: &[String; 3]) -> [RustupCall<'_>; 3] {
    let [declared, stable, nightly] = statuses;
    let [declared_output, stable_output, nightly_output] = outputs;
    [
        RustupCall::new(1, declared, Some(declared_output.as_str())),
        RustupCall::new(2, stable, Some(stable_output.as_str())),
        RustupCall::new(3, nightly, Some(nightly_output.as_str())),
    ]
}

/// Writes every configured response for one fake subcommand.
fn write_rustup_calls(
    command: &mut Command,
    subcommand: &str,
    calls: &[RustupCall<'_>],
) -> Result<(), String> {
    for call in calls {
        set_rustup_ordinal(command, subcommand, *call)?;
    }
    Ok(())
}

/// Writes the status and, when present, the output for one call ordinal.
fn set_rustup_ordinal(
    command: &mut Command,
    subcommand: &str,
    call: RustupCall<'_>,
) -> Result<(), String> {
    command.env(
        format!("GAUSS_RUSTUP_{subcommand}_STATUS_{}", call.ordinal),
        exit_value(call.status)?,
    );
    if let Some(text) = call.output {
        command.env(
            format!("GAUSS_RUSTUP_{subcommand}_OUTPUT_{}", call.ordinal),
            text,
        );
    }
    Ok(())
}

/// Renders a fake exit status, rejecting values a shell cannot report.
fn exit_value(status: i32) -> Result<String, String> {
    if !(0..=255).contains(&status) {
        return Err(format!("fake exit status is outside 0..=255: {status}"));
    }
    Ok(status.to_string())
}
