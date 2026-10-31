//! Contract test for the development build footprint rule.
//!
//! The Concordat `rust-dev-build-footprint` rule needs every default
//! development profile to disable incremental compilation and debug
//! information, first-party package overrides included. The pin is stable, so
//! the native Cargo metadata key does not apply and the Cargo configuration
//! must not carry an `[unstable]` metadata setting, which stable Cargo cannot
//! honour. This contract holds those facts, then proves the profile settings
//! and both documented overrides at the Cargo boundary.

use std::process::Command;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use rstest::rstest;

/// Root manifest holding the development profile.
const MANIFEST: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
/// Cargo configuration whose rustflags the fixture crate inherits.
const CARGO_CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.cargo/config.toml"));
/// Toolchain file naming the stable channel the fixture builds under.
const TOOLCHAIN: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/rust-toolchain.toml"));
/// Variables that would add a wrapper or override the profile under test.
const CLEARED: [&str; 11] = [
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_BUILD_RUSTFLAGS",
    "RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_BUILD_INCREMENTAL",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_DEV_INCREMENTAL",
];

/// Turns a failed condition into the message that explains it.
fn require(condition: bool, message: impl FnOnce() -> String) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message()) }
}

/// Returns the non-comment lines of the named TOML table.
fn table_lines<'a>(document: &'a str, header: &str) -> Vec<&'a str> {
    document
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty())
        .collect()
}

/// Returns the value assigned to `key` in a table's lines.
fn value<'a>(lines: &[&'a str], key: &str) -> Option<&'a str> {
    lines.iter().find_map(|line| {
        let (name, assigned) = line.split_once('=')?;
        (name.trim() == key).then_some(assigned.trim())
    })
}

/// Returns the pinned channel named in `rust-toolchain.toml`, if any.
fn pinned_channel() -> Option<String> {
    TOOLCHAIN.lines().find_map(|line| {
        let (name, assigned) = line.split_once('=')?;
        (name.trim() == "channel").then(|| assigned.trim().trim_matches('"').to_owned())
    })
}

/// Returns whether a `debug` value disables debug information.
fn disables_debug(setting: Option<&str>) -> bool {
    matches!(setting, Some("0" | "false"))
}

/// Scenario: the default development profile.
///
/// Invariant: it keeps no incremental state and no debug information.
#[test]
fn the_development_profile_keeps_the_build_footprint_settings() -> Result<(), String> {
    let profile = table_lines(MANIFEST, "[profile.dev]");
    require(value(&profile, "incremental") == Some("false"), || {
        "profile.dev must set incremental = false".to_owned()
    })?;
    require(disables_debug(value(&profile, "debug")), || {
        format!(
            "profile.dev must set debug = 0 or false, found {:?}",
            value(&profile, "debug")
        )
    })
}

/// Scenario: the per-package overrides of the development profile.
///
/// Invariant: first-party packages keep no debug information either, because a
/// package override replaces the profile default for that package.
#[rstest]
#[case::dependencies("[profile.dev.package.\"*\"]")]
#[case::gauss("[profile.dev.package.gauss]")]
#[case::gauss_core("[profile.dev.package.gauss-core]")]
#[case::gauss_svg("[profile.dev.package.gauss-svg]")]
fn every_development_package_override_disables_debug_info(
    #[case] header: &str,
) -> Result<(), String> {
    let lines = table_lines(MANIFEST, header);
    require(disables_debug(value(&lines, "debug")), || {
        format!(
            "{header} must set debug = 0 or false, found {:?}",
            value(&lines, "debug")
        )
    })
}

/// Scenario: the stable pin and its Cargo configuration.
///
/// Invariant: a stable pin cannot read `[unstable]` metadata keys, so the
/// configuration carries none, and no raw rustc metadata flag appears.
#[test]
fn a_stable_pin_carries_no_unstable_metadata_setting() -> Result<(), String> {
    let channel = pinned_channel().unwrap_or_default();
    require(!channel.starts_with("nightly"), || {
        format!("expected a stable pin, found {channel:?}")
    })?;
    require(!CARGO_CONFIG.contains("embed-metadata"), || {
        "stable Cargo cannot read an embed-metadata setting of either spelling".to_owned()
    })
}

/// Returns the `[profile.dev]` table text of the root manifest.
fn dev_profile() -> Result<String, String> {
    let lines = table_lines(MANIFEST, "[profile.dev]");
    require(!lines.is_empty(), || {
        "Cargo.toml must carry a [profile.dev] table".to_owned()
    })?;
    Ok(format!("[profile.dev]\n{}\n", lines.join("\n")))
}

/// Writes the fixture crate: the repository's profile and Cargo configuration.
fn write_fixture(root: &Utf8PathBuf) -> Result<(), String> {
    let fail = |step: &'static str| move |error: std::io::Error| format!("{step}: {error}");
    Dir::create_ambient_dir_all(root, ambient_authority())
        .map_err(fail("creating the fixture directory"))?;
    let dir = Dir::open_ambient_dir(root, ambient_authority())
        .map_err(fail("opening the fixture directory"))?;
    dir.create_dir_all("src").map_err(fail("creating src"))?;
    dir.create_dir_all(".cargo")
        .map_err(fail("creating .cargo"))?;
    let manifest = format!(
        "[package]\nname = \"footprint\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n{}",
        dev_profile()?
    );
    dir.write("Cargo.toml", manifest)
        .map_err(fail("writing Cargo.toml"))?;
    dir.write("src/lib.rs", "pub fn footprint() {}\n")
        .map_err(fail("writing the library"))?;
    dir.write(".cargo/config.toml", CARGO_CONFIG)
        .map_err(fail("writing the configuration"))
}

/// Builds the fixture crate and returns the rustc command line Cargo ran.
fn compile_fixture(name: &str, extra_env: &[(&str, &str)]) -> Result<String, String> {
    let root = Utf8PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    write_fixture(&root)?;
    let channel = pinned_channel().ok_or("rust-toolchain.toml must name a channel")?;
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{channel}"))
        .args(["build", "--color", "never", "--offline", "-v"])
        .current_dir(root.as_std_path())
        .env("CARGO_TARGET_DIR", root.join("target").as_std_path());
    for variable in CLEARED {
        command.env_remove(variable);
    }
    for (variable, setting) in extra_env {
        command.env(variable, setting);
    }
    let output = command
        .output()
        .map_err(|error| format!("running cargo: {error}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    require(output.status.success(), || {
        format!("cargo build failed:\n{stderr}")
    })?;
    stderr
        .lines()
        .find(|line| line.contains("Running") && line.contains("--crate-name footprint"))
        .map(str::to_owned)
        .ok_or_else(|| format!("no rustc command line in Cargo's output:\n{stderr}"))
}

/// Scenario: a default development build.
///
/// Invariant: rustc gets no debug information and no incremental directory.
#[test]
fn a_default_build_has_no_debug_info_or_incremental_state() -> Result<(), String> {
    let command = compile_fixture("footprint-default", &[])?;
    require(!command.contains("-C debuginfo"), || {
        format!("debug info must be off: {command}")
    })?;
    require(!command.contains("-C incremental"), || {
        format!("incremental must be off: {command}")
    })
}

/// Scenario: each documented override of the footprint defaults.
///
/// Invariant: `CARGO_PROFILE_DEV_DEBUG=true` restores debug information, and
/// `CARGO_INCREMENTAL=1` restores incremental compilation.
#[rstest]
#[case::debug("footprint-debug", "CARGO_PROFILE_DEV_DEBUG", "true", "-C debuginfo=2")]
#[case::incremental("footprint-incremental", "CARGO_INCREMENTAL", "1", "-C incremental=")]
fn a_documented_override_restores_its_setting(
    #[case] name: &str,
    #[case] variable: &str,
    #[case] setting: &str,
    #[case] expected: &str,
) -> Result<(), String> {
    let command = compile_fixture(name, &[(variable, setting)])?;
    require(command.contains(expected), || {
        format!("{variable}={setting} must restore the setting ({expected}): {command}")
    })
}
