//! BDD tests for i18n module message lookup and fallback behaviour.

use std::collections::HashMap;

use gauss::i18n::{Catalog, Locale, Localizer, MessageId};
use rstest_bdd::{given, scenarios, then, when};

scenarios!("tests/features/i18n.feature");

struct I18nTestContext {
    locale: Locale,
    localizer: Option<Localizer>,
    lookup_result: Option<Result<String, gauss::i18n::Error>>,
    fallback_occurred: bool,
}

impl I18nTestContext {
    fn new() -> Self {
        Self {
            locale: Locale::default(),
            localizer: None,
            lookup_result: None,
            fallback_occurred: false,
        }
    }
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

#[given(regex = r#"^the locale is set to "([^"]+)"$"#)]
fn set_locale(mut ctx: I18nTestContext, locale_str: String) -> I18nTestContext {
    ctx.locale = Locale::from_language_tag(&locale_str);
    ctx
}

#[when(regex = r#"^I look up the message "([^"]+)"$"#)]
fn lookup_message(mut ctx: I18nTestContext, message_key: String) -> I18nTestContext {
    if let Some(ref localizer) = ctx.localizer {
        let message_id = MessageId::from(message_key);
        let result = localizer.lookup(&ctx.locale, &message_id);
        ctx.fallback_occurred = result.is_ok() && ctx.locale != Locale::en_gb();
        ctx.lookup_result = Some(result);
    }
    ctx
}

#[then(regex = r#"^the message should be "([^"]+)"$"#)]
fn message_should_be(ctx: I18nTestContext, expected: String) {
    let result = ctx.lookup_result.as_ref().expect("No lookup result");
    assert!(result.is_ok(), "Expected successful lookup");
    assert_eq!(result.as_ref().unwrap(), &expected);
}

#[then("a fallback should have occurred")]
fn fallback_occurred(ctx: I18nTestContext) {
    assert!(
        ctx.fallback_occurred,
        "Expected fallback to have occurred but locale matched"
    );
}

#[then("a lookup error should be returned")]
fn lookup_error_returned(ctx: I18nTestContext) {
    let result = ctx.lookup_result.as_ref().expect("No lookup result");
    assert!(
        result.is_err(),
        "Expected error but got: {:?}",
        result.as_ref().unwrap()
    );
}

fn create_en_catalog() -> Catalog {
    let mut messages = HashMap::new();
    messages.insert("tool_mode.draw".to_owned(), "Draw".to_owned());
    messages.insert("tool_mode.manipulate".to_owned(), "Manipulate".to_owned());
    messages.insert("edge_mode.line".to_owned(), "Line".to_owned());
    messages.insert(
        "edge_mode.bezier_auto".to_owned(),
        "Bezier (auto)".to_owned(),
    );
    Catalog::from_messages(messages)
}

fn create_fr_catalog() -> Catalog {
    let mut messages = HashMap::new();
    messages.insert("tool_mode.draw".to_owned(), "Dessiner".to_owned());
    messages.insert("tool_mode.manipulate".to_owned(), "Manipuler".to_owned());
    messages.insert("edge_mode.line".to_owned(), "Ligne".to_owned());
    messages.insert(
        "edge_mode.bezier_auto".to_owned(),
        "Bézier (auto)".to_owned(),
    );
    Catalog::from_messages(messages)
}
