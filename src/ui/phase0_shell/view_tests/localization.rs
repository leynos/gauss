//! Typed localization and fallback tests for the Phase 0 shell view.
//!
//! These tests force lookup failures through the `set_localizer` test seam and
//! verify that the typed lookup reports errors instead of silently discarding
//! them, while render-boundary helpers use their documented fallbacks.

use std::collections::HashMap;

use gpui::TestAppContext;

use super::{super::lookup_template, setup_phase0_shell, update_shell};
use crate::i18n::{Catalog, I18nError, Locale, Localizer, MessageId};

/// Build a localizer whose en-GB catalogue contains only `entries`.
///
/// Keys absent from `entries` are absent from the sole catalogue, so lookups
/// for them must fail rather than fall back to the default locale.
fn localizer_with_entries(entries: &[(&str, &str)]) -> Localizer {
    let messages = entries
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::en_gb(), Catalog::from_messages(messages));
    Localizer::with_catalogs(catalogs, Locale::en_gb())
}

/// A known key localizes through the injected catalogue.
///
/// The injected value differs from the built-in catalogue so an
/// implementation that bypasses the injected localizer fails the assertion.
#[gpui::test]
fn localize_returns_ok_for_known_key(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    update_shell(visual_cx, &view, |shell| {
        shell.set_localizer(localizer_with_entries(&[(
            "tool_mode.draw",
            "Injected Draw",
        )]));
    });

    let result = visual_cx.read(|app| view.read(app).localize(&MessageId::tool_mode_draw()));
    assert_eq!(
        result,
        Ok("Injected Draw".to_owned()),
        "expected the typed lookup to return the injected catalogue value"
    );
}

/// A key missing from every catalogue surfaces an explicit error.
#[gpui::test]
fn localize_returns_message_not_found_for_missing_key(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    update_shell(visual_cx, &view, |shell| {
        shell.set_localizer(localizer_with_entries(&[]));
    });

    let message_id = MessageId::from("missing.key");
    let result = visual_cx.read(|app| view.read(app).localize(&message_id));
    assert_eq!(
        result,
        Err(I18nError::MessageNotFound {
            message_id: "missing.key".to_owned(),
            locale: "en-gb".to_owned(),
        }),
        "expected the typed lookup to report the missing message"
    );
}

/// The render-boundary helper returns a translation when one exists.
#[gpui::test]
fn localize_or_key_returns_translation_for_known_key(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    update_shell(visual_cx, &view, |shell| {
        shell.set_localizer(localizer_with_entries(&[(
            "tool_mode.draw",
            "Injected Draw",
        )]));
    });

    let label = visual_cx.read(|app| view.read(app).localize_or_key(&MessageId::tool_mode_draw()));
    assert_eq!(
        label, "Injected Draw",
        "expected the helper to return the injected translation, not the key"
    );
}

/// The render-boundary helper falls back to the raw message identifier.
#[gpui::test]
fn localize_or_key_falls_back_to_message_id(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    update_shell(visual_cx, &view, |shell| {
        shell.set_localizer(localizer_with_entries(&[]));
    });

    let message_id = MessageId::from("missing.key");
    let label = visual_cx.read(|app| view.read(app).localize_or_key(&message_id));
    assert_eq!(
        label, "missing.key",
        "expected the fallback to return the raw message identifier"
    );
}

/// Template lookups prefer a catalogue template when one exists.
#[gpui::test]
fn lookup_template_prefers_catalogue_template(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    update_shell(visual_cx, &view, |shell| {
        shell.set_localizer(localizer_with_entries(&[(
            "tool.status.mode",
            "Catalogue: {tool}",
        )]));
    });

    let template = visual_cx.read(|app| {
        lookup_template(
            view.read(app),
            &MessageId::tool_status_mode(),
            "Fallback: {tool}",
        )
    });
    assert_eq!(
        template, "Catalogue: {tool}",
        "expected the template lookup to prefer the catalogue template"
    );
}

/// Template lookups return their caller-supplied fallback on failure.
#[gpui::test]
fn lookup_template_returns_fallback_on_failure(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_phase0_shell(cx);
    update_shell(visual_cx, &view, |shell| {
        shell.set_localizer(localizer_with_entries(&[]));
    });

    let template = visual_cx.read(|app| {
        lookup_template(
            view.read(app),
            &MessageId::from("missing.template"),
            "Mode: {tool}",
        )
    });
    assert_eq!(
        template, "Mode: {tool}",
        "expected the template lookup to use its caller fallback"
    );
}
