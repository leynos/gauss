//! Compile-time coverage for the public i18n facade and module privacy.
//!
//! The fixtures check that clients can use the exported API while the
//! implementation remains inaccessible by its internal module path.

/// Checks the public facade and rejects the private implementation path.
#[test]
fn localizer_public_api_preserves_implementation_privacy() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/localizer_api/public_facade.rs");
    cases.compile_fail("tests/ui/localizer_api/private_module.rs");
}
