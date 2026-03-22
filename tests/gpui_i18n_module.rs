//! GPUI tests for i18n integration with Phase 0 shell and accessibility.

use std::collections::HashMap;

use accesskit::Role;
use gpui::{Context as _, TestAppContext};

use gauss::i18n::{Catalog, Locale, Localizer};
use gauss::model::{EdgeMode, ToolMode};
use gauss::ui::phase0_shell::Phase0Shell;

#[gpui::test]
async fn phase0_shell_status_uses_default_english_catalog(cx: &mut TestAppContext) {
    let window = cx.add_window(|cx| Phase0Shell::new(cx));
    let shell = window.root_view(cx).expect("window should have view");

    let status = shell.read_with(cx, |shell, _cx| shell.mode_status_line());

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
async fn phase0_shell_status_uses_injected_test_catalog(cx: &mut TestAppContext) {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), create_test_french_catalog());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let window = cx.add_window(|cx| {
        let mut shell = Phase0Shell::new(cx);
        shell.set_localizer(localizer.clone());
        shell.set_locale(Locale::fr_fr());
        shell
    });
    let shell = window.root_view(cx).expect("window should have view");

    let status = shell.read_with(cx, |shell, _cx| shell.mode_status_line());

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
async fn a11y_status_node_uses_same_localized_text(cx: &mut TestAppContext) {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), create_test_french_catalog());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let window = cx.add_window(|cx| {
        let mut shell = Phase0Shell::new(cx);
        shell.set_localizer(localizer.clone());
        shell.set_locale(Locale::fr_fr());
        shell
    });

    window.update(cx, |_shell, cx| {
        let tree = cx.window.accesskit_tree.as_ref().expect("AccessKit tree");
        let nodes = &tree.state().nodes;

        let status_node = nodes
            .values()
            .find(|node| node.role() == Role::Status)
            .expect("Status node should exist");

        let label = status_node.label().expect("Status node should have label");
        assert!(
            label.contains("Dessiner"),
            "Expected French 'Dessiner' in a11y status, got: {label}"
        );
        assert!(
            label.contains("Ligne"),
            "Expected French 'Ligne' in a11y status, got: {label}"
        );
    });
}

#[gpui::test]
async fn a11y_status_node_matches_visible_status_line(cx: &mut TestAppContext) {
    let window = cx.add_window(|cx| Phase0Shell::new(cx));
    let shell = window.root_view(cx).expect("window should have view");

    let visible_status = shell.read_with(cx, |shell, _cx| shell.mode_status_line());

    window.update(cx, |_shell, cx| {
        let tree = cx.window.accesskit_tree.as_ref().expect("AccessKit tree");
        let nodes = &tree.state().nodes;

        let status_node = nodes
            .values()
            .find(|node| node.role() == Role::Status)
            .expect("Status node should exist");

        let a11y_label = status_node.label().expect("Status node should have label");

        assert!(
            visible_status.contains("Draw"),
            "Visible status should contain 'Draw'"
        );
        assert!(
            a11y_label.contains("Draw"),
            "A11y status should contain 'Draw'"
        );
        assert!(
            visible_status.contains("Line"),
            "Visible status should contain 'Line'"
        );
        assert!(
            a11y_label.contains("Line"),
            "A11y status should contain 'Line'"
        );
    });
}

fn create_test_french_catalog() -> Catalog {
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
