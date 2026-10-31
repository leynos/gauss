//! Contract test for the development build footprint rule.
//!
//! The Concordat `rust-dev-build-footprint` rule needs every default
//! development profile to disable incremental compilation and debug
//! information, first-party package overrides included. The pin is stable, so
//! the native Cargo metadata key does not apply and the Cargo configuration
//! must not carry an `[unstable]` metadata setting, which stable Cargo cannot
//! honour. This contract holds those facts by reading the configuration and starts
//! no Cargo process.

use rstest::rstest;

/// Root manifest holding the development profile.
const MANIFEST: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
/// Cargo configuration whose rustflags the fixture crate inherits.
const CARGO_CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.cargo/config.toml"));
/// Toolchain file naming the stable channel the fixture builds under.
const TOOLCHAIN: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/rust-toolchain.toml"));
/// A TOML table header, such as `[profile.dev]`.
#[derive(Clone, Copy, Debug)]
struct Table(&'static str);

/// A key within a TOML table.
#[derive(Clone, Copy, Debug)]
struct Key(&'static str);

/// The default development profile.
const PROFILE_DEV: Table = Table("[profile.dev]");
/// The `incremental` key of a profile.
const INCREMENTAL: Key = Key("incremental");
/// The `debug` key of a profile.
const DEBUG: Key = Key("debug");

/// Turns a failed condition into the message that explains it.
fn require(condition: bool, message: impl FnOnce() -> String) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message()) }
}

/// Returns the non-comment lines of the named TOML table.
fn table_lines(document: &str, table: Table) -> Vec<&str> {
    let header = table.0;
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
fn value<'a>(lines: &[&'a str], key: Key) -> Option<&'a str> {
    lines.iter().find_map(|line| {
        let (name, assigned) = line.split_once('=')?;
        (name.trim() == key.0).then_some(assigned.trim())
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
    let profile = table_lines(MANIFEST, PROFILE_DEV);
    require(value(&profile, INCREMENTAL) == Some("false"), || {
        "profile.dev must set incremental = false".to_owned()
    })?;
    require(disables_debug(value(&profile, DEBUG)), || {
        format!(
            "profile.dev must set debug = 0 or false, found {:?}",
            value(&profile, DEBUG)
        )
    })
}

/// Scenario: the per-package overrides of the development profile.
///
/// Invariant: no named first-party package table pins debug information, because a package
/// table outranks the profile and the environment and would defeat
/// `CARGO_PROFILE_DEV_DEBUG=true` for that crate.
#[rstest]
#[case::gauss(Table("[profile.dev.package.gauss]"))]
#[case::gauss_core(Table("[profile.dev.package.gauss-core]"))]
#[case::gauss_svg(Table("[profile.dev.package.gauss-svg]"))]
fn no_development_package_override_pins_debug_info(#[case] table: Table) -> Result<(), String> {
    let lines = table_lines(MANIFEST, table);
    require(value(&lines, DEBUG).is_none(), || {
        format!(
            "{} must not set debug, found {:?}: the environment override could not reach it",
            table.0,
            value(&lines, DEBUG)
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
