//! GPUI tests for i18n integration with Phase 0 shell.

mod common;

use std::collections::HashMap;

use gpui::TestAppContext;

use gauss::i18n::{Catalog, Locale, Localizer};
use gauss::ui::phase0_shell::Phase0Shell;

use common::{init_test_app, test_french_catalog};

#[gpui::test]
fn phase0_shell_status_uses_default_english_catalog(cx: &mut TestAppContext) {
    init_test_app(cx);
    let (shell, _visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));

    let status = shell.read_with(cx, |s, _cx| s.mode_status_line_for_tests());

    assert!(
        status.contains("Draw"),
        "Expected English 'Draw' in status line, got: {status}"
    );
    assert!(
        status.contains("Line"),
        "Expected English 'Line' in status line, got: {status}"
    );
}

#[gpui::test]
fn phase0_shell_status_uses_injected_test_catalog(cx: &mut TestAppContext) {
    init_test_app(cx);
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), test_french_catalog());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let (shell, _visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell_instance = Phase0Shell::new(view_cx);
        shell_instance.set_localizer(localizer.clone());
        shell_instance.set_locale(Locale::fr_fr());
        shell_instance
    });

    let status = shell.read_with(cx, |s, _cx| s.mode_status_line_for_tests());

    assert!(
        status.contains("Dessiner"),
        "Expected French 'Dessiner' in status line, got: {status}"
    );
    assert!(
        status.contains("Ligne"),
        "Expected French 'Ligne' in status line, got: {status}"
    );
}

#[gpui::test]
fn locale_switching_updates_status_line(cx: &mut TestAppContext) {
    init_test_app(cx);
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), test_french_catalog());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let (shell, _visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell_instance = Phase0Shell::new(view_cx);
        shell_instance.set_localizer(localizer.clone());
        shell_instance
    });

    // Verify English default
    let status_en = shell.read_with(cx, |s, _cx| s.mode_status_line_for_tests());
    assert!(
        status_en.contains("Draw"),
        "Expected English 'Draw' before locale switch, got: {status_en}"
    );

    // Switch to French
    shell.update(cx, |s, _cx| {
        s.set_locale(Locale::fr_fr());
    });

    // Verify French after switch
    let status_fr = shell.read_with(cx, |s, _cx| s.mode_status_line_for_tests());
    assert!(
        status_fr.contains("Dessiner"),
        "Expected French 'Dessiner' after locale switch, got: {status_fr}"
    );
    assert!(
        status_fr.contains("Ligne"),
        "Expected French 'Ligne' after locale switch, got: {status_fr}"
    );
}

#[gpui::test]
fn fallback_to_default_locale_when_message_missing(cx: &mut TestAppContext) {
    init_test_app(cx);
    // Create a French catalog with only one message (missing the edge mode messages)
    let mut messages = HashMap::new();
    messages.insert("tool_mode.draw".to_owned(), "Dessiner".to_owned());
    let partial_catalog = Catalog::from_messages(messages);

    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), partial_catalog);
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let (shell, _visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell_instance = Phase0Shell::new(view_cx);
        shell_instance.set_localizer(localizer.clone());
        shell_instance.set_locale(Locale::fr_fr());
        shell_instance
    });

    let status = shell.read_with(cx, |s, _cx| s.mode_status_line_for_tests());

    // Should have French tool mode
    assert!(
        status.contains("Dessiner"),
        "Expected French 'Dessiner' from partial catalog, got: {status}"
    );

    // Should fall back to English for missing edge mode
    assert!(
        status.contains("Line"),
        "Expected English fallback 'Line' for missing message, got: {status}"
    );
}

#[gpui::test]
fn manipulate_mode_omits_edge_mode_in_status_line(cx: &mut TestAppContext) {
    use gauss::model::ToolMode;

    init_test_app(cx);
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), test_french_catalog());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let (shell, _visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell_instance = Phase0Shell::new(view_cx);
        shell_instance.set_localizer(localizer.clone());
        shell_instance.set_locale(Locale::fr_fr());
        shell_instance
    });

    // Switch to Manipulate mode
    shell.update(cx, |s, _cx| {
        s.set_tool_mode_for_tests(ToolMode::Manipulate);
    });

    let status = shell.read_with(cx, |s, _cx| s.mode_status_line_for_tests());

    // Should contain localized tool mode
    assert!(
        status.contains("Manipuler"),
        "Expected French 'Manipuler' for Manipulate mode, got: {status}"
    );

    // Should NOT contain edge mode (neither French nor English)
    assert!(
        !status.contains("Ligne") && !status.contains("Line"),
        "Manipulate mode should omit edge mode fragment, got: {status}"
    );
    assert!(
        !status.contains("Bézier") && !status.contains("Bezier"),
        "Manipulate mode should omit edge mode fragment, got: {status}"
    );
}
