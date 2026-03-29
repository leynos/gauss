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
fn default_catalog_exists(ctx: &mut I18nTestContext) {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::en_gb(), create_en_catalog());
    ctx.localizer = Some(Localizer::with_catalogs(catalogs, Locale::en_gb()));
}

#[given("a test French catalog is available for testing")]
fn french_catalog_available(ctx: &mut I18nTestContext) {
    if let Some(ref mut localizer) = ctx.localizer {
        localizer.add_catalog(Locale::fr_fr(), test_french_catalog());
    }
}

#[given("the locale is set to {locale_str}")]
fn set_locale(ctx: &mut I18nTestContext, locale_str: String) {
    // Strip quotes from the locale string (feature file includes quotes)
    let locale_tag = locale_str.trim_matches('"');
    ctx.locale = Locale::from_language_tag(locale_tag);
}

#[when("I look up the message {message_key}")]
fn lookup_message(ctx: &mut I18nTestContext, message_key: String) {
    if let Some(ref localizer) = ctx.localizer {
        // Strip quotes from the message key (feature file includes quotes)
        let key = message_key.trim_matches('"');
        let message_id = MessageId::from(key.to_owned());

        // Check if the requested locale's catalog contains the message
        let requested_catalog_has_message = localizer.catalog_contains(&ctx.locale, &message_id);

        // Perform the lookup (which may fall back to default locale)
        let result = localizer.lookup(&ctx.locale, &message_id);

        // Fallback occurred if lookup succeeded but the requested catalog didn't have it
        ctx.fallback_occurred = result.is_ok() && !requested_catalog_has_message;
        ctx.lookup_result = Some(result);
    }
}

#[then("the message should be {expected}")]
#[expect(
    clippy::expect_used,
    reason = "Test code - panic on missing result is acceptable"
)]
#[expect(
    clippy::unwrap_used,
    reason = "Test code - unwrap after assert is safe"
)]
fn message_should_be(ctx: &I18nTestContext, expected: String) {
    let result = ctx
        .lookup_result
        .as_ref()
        .expect("No lookup result: lookup_message step must run first");
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
fn fallback_occurred(ctx: &I18nTestContext) {
    assert!(
        ctx.fallback_occurred,
        "Expected fallback to have occurred but locale matched"
    );
}

#[then("a lookup error should be returned")]
#[expect(
    clippy::expect_used,
    reason = "Test code - panic on missing result is acceptable"
)]
fn lookup_error_returned(ctx: &I18nTestContext) {
    let result = ctx
        .lookup_result
        .as_ref()
        .expect("No lookup result: lookup_message step must run first");
    assert!(result.is_err(), "Expected error but got: {result:?}");
}

/// Creates a test catalog with the given key-value pairs.
fn make_test_catalog(entries: &[(&str, &str)]) -> Catalog {
    let mut messages = HashMap::new();
    for (key, value) in entries {
        messages.insert((*key).to_owned(), (*value).to_owned());
    }
    Catalog::from_messages(messages)
}

/// Creates a French test catalog with common UI messages.
fn test_french_catalog() -> Catalog {
    make_test_catalog(&[
        ("tool_mode.draw", "Dessiner"),
        ("tool_mode.manipulate", "Manipuler"),
        ("edge_mode.line", "Ligne"),
        ("edge_mode.bezier_auto", "Bézier (auto)"),
    ])
}

fn create_en_catalog() -> Catalog {
    Catalog::default_en_gb()
}

// Test functions for each scenario in the feature file
#[scenario(
    path = "tests/features/i18n.feature",
    name = "Successful message lookup in default catalog"
)]
fn successful_lookup_in_default_catalog(ctx: I18nTestContext) {
    let _ = ctx; // Use the fixture
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Successful message lookup in alternate catalog"
)]
fn successful_lookup_in_alternate_catalog(ctx: I18nTestContext) {
    let _ = ctx;
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Fallback to English when locale is unsupported"
)]
fn fallback_to_english_unsupported_locale(ctx: I18nTestContext) {
    let _ = ctx;
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Error when message key is unavailable"
)]
fn error_for_unavailable_key(ctx: I18nTestContext) {
    let _ = ctx;
}

#[scenario(
    path = "tests/features/i18n.feature",
    name = "Edge mode message lookup"
)]
fn edge_mode_message_lookup(ctx: I18nTestContext) {
    let _ = ctx;
}
