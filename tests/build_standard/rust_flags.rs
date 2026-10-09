//! Gauss composes its own `RUST_FLAGS` variable into the `RUSTFLAGS` its recipes assign. A caller who
//! overrides it on the `make` command line must still see the override reach every recipe that
//! builds, tests or lints, beside the caller's inherited `RUSTFLAGS` and the standard flags.

use std::process::Command as Process;

use super::make::{Command, Host, commands_with_text};

/// A distinctive value no recipe or configuration contains by default.
const MARKER: &str = "--cfg=gauss_rust_flags_marker";

/// The development targets that compose `RUST_FLAGS`, and how many commands of each carry it.
const COMPOSING_TARGETS: &[(&str, usize)] = &[
    ("test", 3),
    ("test-ci", 1),
    ("test-quick", 1),
    ("lint", 2),
    ("typecheck", 1),
];

/// Whether a command is one that builds, tests or lints and so takes `RUST_FLAGS`.
fn composes_rust_flags(text: &str) -> bool {
    [
        "nextest run",
        "cargo test",
        "cargo check",
        "clippy",
        "whitaker",
    ]
    .iter()
    .any(|tool| text.contains(tool))
}

/// Returns the complaints about the commands of a target: each command that composes `RUST_FLAGS` must
/// carry the marker, and a `RUSTFLAGS` assignment must keep the caller's own flags before it.
fn problems(target: &str, commands: &[Command], expected: usize) -> Vec<String> {
    let composing: Vec<&Command> = commands
        .iter()
        .filter(|command| composes_rust_flags(&command.text))
        .collect();
    let mut found = Vec::new();
    if composing.len() != expected {
        found.push(format!(
            "`make {target}` has {} commands composing RUST_FLAGS, not {expected}",
            composing.len()
        ));
    }
    for command in composing {
        if !command.text.contains(MARKER) {
            found.push(format!(
                "`make {target}` drops the RUST_FLAGS override: {}",
                command.text
            ));
        } else if command.text.contains("RUSTFLAGS=")
            && !command.text.contains("${RUSTFLAGS:+$RUSTFLAGS }")
        {
            found.push(format!(
                "`make {target}` drops the caller's RUSTFLAGS: {}",
                command.text
            ));
        }
    }
    found
}

/// Reads the commands `make -n` prints for a target with the override on its command line.
fn commands_for(target: &str, host: Host, caller: Option<&str>) -> Result<Vec<Command>, String> {
    let mut process = Process::new("make");
    process
        .args([
            "-n",
            "-B",
            &format!("BUILD_HOST_OS={}", host.make_value()),
            &format!("RUST_FLAGS={MARKER}"),
            target,
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    if let Some(flags) = caller {
        process.env("RUSTFLAGS", flags);
    }
    let output = process
        .output()
        .map_err(|error| format!("running make: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`make -n {target}` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("`make -n {target}` is not UTF-8: {error}"))?;
    commands_with_text(&stdout)
}

#[test]
fn a_rust_flags_override_reaches_every_recipe_that_composes_it() -> Result<(), String> {
    let mut found = Vec::new();
    for host in [Host::Linux, Host::Darwin] {
        for caller in [None, Some(""), Some("-C debuginfo=0")] {
            for (target, expected) in COMPOSING_TARGETS {
                found.extend(problems(
                    target,
                    &commands_for(target, host, caller)?,
                    *expected,
                ));
            }
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!("{found:#?}"))
    }
}

mod fixtures {
    //! The override check over fixtures: a compliant recipe passes, a dropped override, a dropped
    //! caller's flags and a missing command are each refused.

    use super::super::make::commands_with_text;
    use super::{MARKER, problems};

    fn run(text: &str, expected: usize) -> usize {
        let commands = commands_with_text(text).unwrap_or_default();
        problems("test", &commands, expected).len()
    }

    fn compliant() -> String {
        format!(
            "RUSTFLAGS=\"${{RUSTFLAGS:+$RUSTFLAGS }}{MARKER} -Clink-arg=-fuse-ld=mold\" cargo nextest run\n"
        )
    }

    #[test]
    fn a_compliant_recipe_passes() {
        assert_eq!(run(&compliant(), 1), 0);
    }

    #[test]
    fn a_dropped_override_is_refused() {
        assert_eq!(run(&compliant().replace(MARKER, "-D warnings"), 1), 1);
    }

    #[test]
    fn dropped_caller_flags_are_refused() {
        assert_eq!(
            run(&compliant().replace("${RUSTFLAGS:+$RUSTFLAGS }", ""), 1),
            1
        );
    }

    #[test]
    fn a_missing_command_is_refused() {
        assert_eq!(run(&compliant(), 2), 1);
        assert_eq!(run("echo hi\n", 1), 1);
    }
}
