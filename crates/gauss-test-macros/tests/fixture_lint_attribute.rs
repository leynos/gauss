//! Compile-time coverage for the fixture lint attribute.
//!
//! Both directions are retained. The pass fixture denies `unused_braces`, so it
//! compiles only while `allow_fixture_expansion_lints` scopes the allowance
//! around the compacted `rstest` fixture expansion. The fail fixture is the
//! same source without the attribute, and it must report `unused_braces`, which
//! pins the diagnostic the attribute exists to suppress.

#[test]
fn lint_attribute_governs_the_expansion() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/compacted_fixture.rs");
    cases.compile_fail("tests/ui/compacted_fixture_without_attribute.rs");
}
