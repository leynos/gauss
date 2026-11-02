//! Direct Cargo route classification cases.

use rstest::rstest;

use super::super::{Route, make_tools};

/// Direct Cargo suite classification refuses nonexecuting switches and unknown subcommands.
#[rstest]
#[case::make_test_help(&["cargo", "test", "--help"], "--help")]
#[case::make_test_no_run(&["cargo", "test", "--no-run"], "--no-run")]
#[case::make_test_list(&["cargo", "test", "--", "--list"], "--list")]
#[case::make_nextest_list(&["cargo", "nextest", "list"], "unresolved Cargo subcommand: nextest")]
fn cargo_nonexecution_modifiers_are_gaps(#[case] words: &[&str], #[case] diagnostic: &str) {
    let baseline = ["cargo", "test", "--workspace"].map(str::to_owned);
    assert_eq!(
        make_tools::classify_words(&baseline, false),
        Ok(Route::Suite)
    );
    let mutated = words
        .iter()
        .map(|word| (*word).to_owned())
        .collect::<Vec<_>>();
    let error = make_tools::classify_words(&mutated, false)
        .expect_err("nonexecuting or unknown Cargo route must not count as a suite");
    assert!(
        error.contains(diagnostic),
        "wrong Cargo route diagnostic: {error}"
    );
}
