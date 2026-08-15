//! Compile-time coverage for the scenario-state macro's visibility form.
//!
//! The fixture imports the macro's test-only source and verifies that its
//! `pub(super)` form exposes each generated helper to the parent test binary.

#[test]
fn visibility_form_compiles() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/scenario_state_visibility.rs");
}
