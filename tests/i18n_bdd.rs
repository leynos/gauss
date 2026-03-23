//! BDD tests for i18n module message lookup and fallback behaviour.

use std::collections::HashMap;

use gauss::i18n::{Catalog, I18nError, Locale, Localizer, MessageId};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};

#[derive(Clone, Default)]
struct I18nTestContext {
    locale: Locale,
    localizer: Option<Localizer>,
    lookup_result: Option<Result<String, I18nError>>,
    fallback_occurred: bool,
}

#[fixture]
fn ctx() -> I18nTestContext {
    I18nTestContext::default()
}

#[given("a default English catalog exists")]
fn default_catalog_exists(mut ctx: I18nTestContext) -> I18nTestContext {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::en_gb(), create_en_catalog());
    ctx.localizer = Some(Localizer::with_catalogs(catalogs, Locale::en_gb()));
    ctx
}

#[given("a test French catalog is available for testing")]
fn french_catalog_available(mut ctx: I18nTestContext) -> I18nTestContext {
    if let Some(ref mut localizer) = ctx.localizer {
        localizer.add_catalog(Locale::fr_fr(), create_fr_catalog());
    }
    ctx
}

#[given("the locale is set to {locale_str}")]
fn set_locale(mut ctx: I18nTestContext, locale_str: String) -> I18nTestContext {
    // Strip quotes from the locale string (feature file includes quotes)
    let locale_tag = locale_str.trim_matches('"');
    ctx.locale = Locale::from_language_tag(locale_tag);
    ctx
}

#[when("I look up the message {message_key}")]
fn lookup_message(mut ctx: I18nTestContext, message_key: String) -> I18nTestContext {
    if let Some(ref localizer) = ctx.localizer {
        // Strip quotes from the message key (feature file includes quotes)
        let key = message_key.trim_matches('"');
        let message_id = MessageId::from(key.to_owned());
        let result = localizer.lookup(&ctx.locale, &message_id);
        ctx.fallback_occurred = result.is_ok() && ctx.locale != Locale::en_gb();
        ctx.lookup_result = Some(result);
    }
    ctx
}

#[then("the message should be {expected}")]
#[expect(clippy::expect_used, reason = "Test code - panic on missing result is acceptable")]
#[expect(clippy::unwrap_used, reason = "Test code - unwrap after assert is safe")]
fn message_should_be(ctx: I18nTestContext, expected: String) {
    let result = ctx.lookup_result.as_ref().expect("No lookup result");
    // Strip quotes from the expected value (feature file includes quotes)
    let expected_value = expected.trim_matches('"');
    assert!(
        result.is_ok(),
        "Expected successful lookup, but got error: {:?}",
        result.as_ref().err()
    );
    assert_eq!(result.as_ref().unwrap(), expected_value);
}

#[then("a fallback should have occurred")]
fn fallback_occurred(ctx: I18nTestContext) {
    assert!(
        ctx.fallback_occurred,
        "Expected fallback to have occurred but locale matched"
    );
}

#[then("a lookup error should be returned")]
#[expect(clippy::expect_used, reason = "Test code - panic on missing result is acceptable")]
#[expect(clippy::unwrap_used, reason = "Test code - unwrap in error message is acceptable")]
fn lookup_error_returned(ctx: I18nTestContext) {
    let result = ctx.lookup_result.as_ref().expect("No lookup result");
    assert!(
        result.is_err(),
        "Expected error but got: {:?}",
        result.as_ref().unwrap()
    );
}

fn make_catalog(entries: &[(&str, &str)]) -> Catalog {
    let messages = entries
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    Catalog::from_messages(messages)
}

fn create_en_catalog() -> Catalog {
    make_catalog(&[
        ("tool_mode.draw", "Draw"),
        ("tool_mode.manipulate", "Manipulate"),
        ("edge_mode.line", "Line"),
        ("edge_mode.bezier_auto", "Bezier (auto)"),
    ])
}

fn create_fr_catalog() -> Catalog {
    make_catalog(&[
        ("tool_mode.draw", "Dessiner"),
        ("tool_mode.manipulate", "Manipuler"),
        ("edge_mode.line", "Ligne"),
        ("edge_mode.bezier_auto", "Bézier (auto)"),
    ])
}

// Test functions for each scenario in the feature file
#[scenario(
    path = "tests/features/i18n.feature",
    name = "Successful message lookup in default catalog"
)]
#[test]
fn successful_lookup_in_default_catalog(ctx: I18nTestContext) {
    let _ = ctx; // Use the fixture
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Successful message lookup in alternate catalog"
)]
#[test]
fn successful_lookup_in_alternate_catalog(ctx: I18nTestContext) {
    let _ = ctx;
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Fallback to English when locale is unsupported"
)]
#[test]
fn fallback_to_english_unsupported_locale(ctx: I18nTestContext) {
    let _ = ctx;
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Error when message key is unavailable"
)]
#[test]
fn error_for_unavailable_key(ctx: I18nTestContext) {
    let _ = ctx;
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Edge mode message lookup"
)]
#[test]
fn edge_mode_message_lookup(ctx: I18nTestContext) {
    let _ = ctx;
}
