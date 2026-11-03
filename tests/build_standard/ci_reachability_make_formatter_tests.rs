//! Proves the production formatter goals and the pinned-prefix invariants.
//!
//! The Make goals are evaluated through the same controlled dry-run harness the
//! workflow checks use, so the complete Cargo argv is proved at the target
//! boundary. The properties cover the unbounded prefix space the classifier
//! must leave fail-closed.

use proptest::prelude::*;

use super::super::{Host, Route, cargo_rows, evaluate_current, make_tools};

/// The complete Cargo argv the production `fmt` goal must reach.
const FMT_ARGV: &[&str] = &[
    make_tools::PROBE_CARGO,
    "+nightly-2026-08-07",
    "fmt",
    "--all",
];

/// The complete Cargo argv the production `check-fmt` goal must reach.
const CHECK_FMT_ARGV: &[&str] = &[
    make_tools::PROBE_CARGO,
    "+nightly-2026-08-07",
    "fmt",
    "--all",
    "--",
    "--check",
];

/// Copies an expected argv slice into owned words for exact comparison.
fn owned(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

/// Returns the single Cargo invocation an evaluated goal reached.
fn single_cargo_argv(goal: &str) -> Vec<String> {
    let evaluation =
        evaluate_current(goal, Host::Linux).expect("the production goal must be evaluable");
    assert_eq!(evaluation.route, Route::NonSuite, "{goal} is not a suite");
    let rows = cargo_rows(&evaluation);
    assert_eq!(
        rows.len(),
        1,
        "{goal} must reach exactly one Cargo invocation"
    );
    rows.first()
        .map(|row| row.words.clone())
        .unwrap_or_default()
}

/// `make fmt` rewrites the workspace with the pinned nightly formatter.
#[test]
fn make_fmt_reaches_the_pinned_formatter_rewrite() {
    assert_eq!(
        single_cargo_argv("fmt"),
        owned(FMT_ARGV),
        "make fmt must select the pinned formatter exactly"
    );
}

/// `make check-fmt` verifies formatting with the pinned nightly formatter.
#[test]
fn make_check_fmt_reaches_the_pinned_formatter_check() {
    assert_eq!(
        single_cargo_argv("check-fmt"),
        owned(CHECK_FMT_ARGV),
        "make check-fmt must select the pinned formatter exactly"
    );
}

proptest! {
    /// Every Cargo prefix other than the exact pin stays an unresolved subcommand.
    #[test]
    fn only_the_exact_pinned_formatter_prefix_is_stripped(
        prefix in "\\+[A-Za-z0-9._-]{0,24}",
        tail in proptest::collection::vec("[A-Za-z0-9._-]{1,12}", 0..4),
    ) {
        prop_assume!(prefix != format!("+{}", make_tools::RUSTFMT_TOOLCHAIN));
        let mut words = vec!["cargo".to_owned(), prefix.clone()];
        words.extend(tail);
        words.push("fmt".to_owned());
        prop_assert_eq!(
            make_tools::classify_words(&words, false),
            Err(format!("unresolved Cargo subcommand: {prefix}"))
        );
    }

    /// The exact pin is stripped so the formatter subcommand is reached.
    #[test]
    fn the_exact_pinned_formatter_prefix_is_stripped(
        tail in proptest::collection::vec("[A-Za-z0-9._-]{1,12}", 0..4),
    ) {
        let mut words = vec![
            "cargo".to_owned(),
            format!("+{}", make_tools::RUSTFMT_TOOLCHAIN),
            "fmt".to_owned(),
        ];
        words.extend(tail);
        prop_assert_eq!(make_tools::classify_words(&words, false), Ok(Route::NonSuite));
    }

    /// Only the exact pinned Rustup installation is accepted.
    #[test]
    fn only_the_exact_pinned_rustup_install_is_accepted(
        arguments in proptest::collection::vec("[A-Za-z0-9._-]{1,12}", 0..8),
    ) {
        let pinned = [
            "toolchain",
            "install",
            make_tools::RUSTFMT_TOOLCHAIN,
            "--profile",
            "minimal",
            "--component",
            "rustfmt",
        ];
        let mut words = vec!["rustup".to_owned()];
        words.extend(arguments.iter().cloned());
        let expected = if arguments.iter().map(String::as_str).eq(pinned) {
            Ok(Route::NonSuite)
        } else {
            Err("unresolved rustup arguments".to_owned())
        };
        prop_assert_eq!(make_tools::classify_words(&words, false), expected);
    }
}
