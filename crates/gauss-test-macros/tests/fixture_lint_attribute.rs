//! Compile-time coverage for the fixture lint attribute.
//!
//! The pass fixture denies `unused_braces`, so it compiles only while
//! `allow_fixture_expansion_lints` scopes the allowance around the compacted
//! `rstest` fixture expansion. Removing the attribute makes the case fail,
//! which is what keeps the workaround honest.

#[test]
fn compacted_fixture_keeps_warnings_denied() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/compacted_fixture.rs");
}
