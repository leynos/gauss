//! Gauss composes its own `RUST_FLAGS` variable into the `RUSTFLAGS` its recipes assign. A caller who
//! overrides it on the `make` command line must still see the override reach every recipe that
//! builds, tests or lints, beside the caller's inherited `RUSTFLAGS` and the standard flags.

use std::process::Command as Process;

use super::make::{Command, Host, commands_with_text};
use super::make_input_env::{COMPETING_RUST_FLAGS, Invocation, NEXTEST_OPTIONS, arguments};

/// A distinctive value no recipe or configuration contains by default.
const MARKER: &str = "--cfg=gauss_rust_flags_marker";
/// The warning-denying input and unique marker supplied to controlled Make runs.
const RUST_FLAGS_INPUT: &str = "-D warnings --cfg=gauss_rust_flags_marker";

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
            &format!("RUST_FLAGS={RUST_FLAGS_INPUT}"),
            "CARGO=probe-cargo",
            "WHITAKER=probe-whitaker",
            "CARGO_FLAGS=--workspace --all-targets --all-features",
            "TEST_FLAGS=--workspace --all-targets --all-features",
            "BUILD_JOBS=-j1",
            "RUSTDOC_FLAGS=--cfg docsrs -D warnings",
            "NEXTEST_FLAGS=",
            target,
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    for name in [
        "MAKEFLAGS",
        "MAKEOVERRIDES",
        "MAKEFILES",
        "GNUMAKEFLAGS",
        "MFLAGS",
        "RUST_FLAGS",
        "RUSTFLAGS",
        "CARGO",
        "WHITAKER",
        "CARGO_FLAGS",
        "TEST_FLAGS",
        "BUILD_JOBS",
        "RUSTDOC_FLAGS",
        "NEXTEST_FLAGS",
    ] {
        process.env_remove(name);
    }
    process.env("RUST_FLAGS", COMPETING_RUST_FLAGS);
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

/// Checks every recorded invocation field and names the first differing contract dimension.
pub(super) fn require_calls(
    expected: &[Invocation],
    actual: &[Invocation],
    context: &str,
) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!(
            "{context} invocation count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, (expected_row, actual_row)) in expected.iter().zip(actual).enumerate() {
        if expected_row.arguments != actual_row.arguments {
            return Err(format!(
                "{context} argv mismatch at invocation {}: expected {:?}, found {:?}",
                index + 1,
                expected_row.arguments,
                actual_row.arguments
            ));
        }
        if expected_row.rust_flags_present != actual_row.rust_flags_present {
            return Err(format!(
                "{context} RUST_FLAGS presence mismatch at invocation {}",
                index + 1
            ));
        }
        if !same_compiler_flag_tokens(&expected_row.rustflags, &actual_row.rustflags) {
            return Err(format!(
                "{context} RUSTFLAGS mismatch at invocation {}: expected {:?}, found {:?}",
                index + 1,
                expected_row.rustflags,
                actual_row.rustflags
            ));
        }
        if !same_compiler_flag_tokens(&expected_row.rustdocflags, &actual_row.rustdocflags) {
            return Err(format!(
                "{context} RUSTDOCFLAGS mismatch at invocation {}",
                index + 1
            ));
        }
    }
    Ok(())
}

/// Compares compiler flag tokens while treating ASCII whitespace as separators.
///
/// For example, `same_compiler_flag_tokens("--cfg a", " --cfg   a ")` is true, while changing
/// `a` to `b` returns false.
fn same_compiler_flag_tokens(expected: &str, actual: &str) -> bool {
    expected
        .split_ascii_whitespace()
        .eq(actual.split_ascii_whitespace())
}

/// Builds the one precise baseline-check diagnostic a named mutation may produce.
pub(super) fn mutation_diagnostic(
    mutation: &str,
    baseline: &[Invocation],
    observed: &[Invocation],
) -> Result<String, String> {
    let (index, field) = match mutation {
        "unexport" => (0, "presence"),
        "nextest-option" => (0, "argv"),
        "fallback-leak" => (1, "argv"),
        "restatement" => (0, "rustflags"),
        _ => return Err(format!("unknown diagnostic for mutation {mutation}")),
    };
    let expected = baseline
        .get(index)
        .ok_or_else(|| format!("{mutation} baseline omitted invocation {}", index + 1))?;
    let actual = observed
        .get(index)
        .ok_or_else(|| format!("{mutation} observation omitted invocation {}", index + 1))?;
    Ok(match field {
        "presence" => format!(
            "{mutation} RUST_FLAGS presence mismatch at invocation {}",
            index + 1
        ),
        "argv" => format!(
            "{mutation} argv mismatch at invocation {}: expected {:?}, found {:?}",
            index + 1,
            expected.arguments,
            actual.arguments
        ),
        "rustflags" => format!(
            "{mutation} RUSTFLAGS mismatch at invocation {}: expected {:?}, found {:?}",
            index + 1,
            expected.rustflags,
            actual.rustflags
        ),
        _ => return Err(format!("unsupported diagnostic field for {mutation}")),
    })
}

/// Applies the presence-field delta expected when `unexport` is removed.
fn unexport_mutation_delta(expected: &mut [Invocation]) -> Result<(), String> {
    expected
        .first_mut()
        .ok_or_else(|| "missing test-ci call".to_owned())?
        .rust_flags_present = true;
    Ok(())
}

/// Removes the two controlled Nextest options expected by that source mutation.
fn nextest_option_mutation_delta(expected: &mut [Invocation]) -> Result<(), String> {
    let argv = &mut expected
        .first_mut()
        .ok_or_else(|| "missing nextest call".to_owned())?
        .arguments;
    let count = argv
        .iter()
        .filter(|arg| arg.as_str() == "--cargo-verbose")
        .count();
    if count != 2 {
        return Err(format!("expected two Nextest options, found {count}"));
    }
    argv.retain(|arg| arg != "--cargo-verbose");
    Ok(())
}

/// Adds the controlled Nextest options at the fallback command's job argument.
fn fallback_leak_mutation_delta(expected: &mut [Invocation]) -> Result<(), String> {
    let argv = &mut expected
        .get_mut(1)
        .ok_or_else(|| "missing fallback call".to_owned())?
        .arguments;
    let at = argv
        .iter()
        .position(|arg| arg == "-j1")
        .ok_or_else(|| "fallback omitted -j1".to_owned())?;
    argv.splice(at..at, arguments(NEXTEST_OPTIONS));
    Ok(())
}

/// Removes the compiler-flag addition expected by the `restatement` mutation.
fn restatement_mutation_delta(expected: &mut [Invocation]) -> Result<(), String> {
    let flags = &mut expected
        .first_mut()
        .ok_or_else(|| "missing typecheck call".to_owned())?
        .rustflags;
    *flags = replace_once(flags, " -D warnings --cfg=gauss_rust_flags_marker", "")?;
    Ok(())
}

/// Confirms the baseline mismatch contains only the named source mutation's exact recorder delta.
pub(super) fn exact_mutation_delta(
    mutation: &str,
    baseline: &[Invocation],
    observed: &[Invocation],
) -> Result<(), String> {
    let mut expected = baseline.to_vec();
    match mutation {
        "unexport" => unexport_mutation_delta(&mut expected)?,
        "nextest-option" => nextest_option_mutation_delta(&mut expected)?,
        "fallback-leak" => fallback_leak_mutation_delta(&mut expected)?,
        "restatement" => restatement_mutation_delta(&mut expected)?,
        _ => return Err(format!("unknown mutation {mutation}")),
    }
    require_calls(&expected, observed, &format!("{mutation} mutation delta"))
}

/// Replaces one unique source fragment, refusing ambiguous mutation points.
pub(super) fn replace_once(source: &str, from: &str, to: &str) -> Result<String, String> {
    let count = source.matches(from).count();
    if count != 1 {
        return Err(format!(
            "expected one mutation match, found {count}: {from}"
        ));
    }
    Ok(source.replacen(from, to, 1))
}

/// Controlled source-copy contracts for Make flag and Nextest routing.
#[cfg(test)]
#[path = "rust_flags_cases.rs"]
pub(super) mod cases;

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
