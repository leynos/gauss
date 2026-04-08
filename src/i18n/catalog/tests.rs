//! Unit tests for i18n catalog and localizer functionality.

use std::collections::HashMap;

use rstest::{fixture, rstest};

use super::*;

#[test]
fn catalog_from_messages_creates_correctly() {
    let mut messages = HashMap::new();
    messages.insert("test".to_owned(), "value".to_owned());
    let catalog = Catalog::from_messages(messages);
    assert_eq!(catalog.get(&MessageId::from("test")), Some("value"));
}

#[test]
fn catalog_get_returns_none_for_missing_key() {
    let catalog = Catalog::from_messages(HashMap::new());
    assert_eq!(catalog.get(&MessageId::from("missing")), None);
}

#[test]
fn catalog_default_en_gb_contains_tool_modes() {
    let catalog = Catalog::default_en_gb();
    assert_eq!(catalog.get(&MessageId::tool_mode_draw()), Some("Draw"));
    assert_eq!(
        catalog.get(&MessageId::tool_mode_manipulate()),
        Some("Manipulate")
    );
}

#[test]
fn catalog_default_en_gb_contains_edge_modes() {
    let catalog = Catalog::default_en_gb();
    assert_eq!(catalog.get(&MessageId::edge_mode_line()), Some("Line"));
    assert_eq!(
        catalog.get(&MessageId::edge_mode_bezier_auto()),
        Some("Bezier (auto)")
    );
}

#[test]
fn localizer_new_creates_with_default_catalog() {
    let localizer = Localizer::new();
    let result = localizer.lookup(&Locale::en_gb(), &MessageId::tool_mode_draw());
    assert!(result.is_ok());
    assert_eq!(result.expect("Should have found message"), "Draw");
}

#[fixture]
fn fr_test_catalog() -> Catalog {
    let mut messages = HashMap::new();
    messages.insert("test".to_owned(), "test_fr".to_owned());
    Catalog::from_messages(messages)
}

#[fixture]
fn fr_test_localizer(fr_test_catalog: Catalog) -> Localizer {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), fr_test_catalog);
    catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
    Localizer::with_catalogs(catalogs, Locale::en_gb())
}

#[rstest]
fn localizer_lookup_succeeds_for_available_locale(fr_test_localizer: Localizer) {
    let result = fr_test_localizer.lookup(&Locale::fr_fr(), &MessageId::from("test"));
    assert!(result.is_ok());
    assert_eq!(result.expect("Should have found message"), "test_fr");
}

#[test]
fn localizer_lookup_falls_back_to_default_locale() {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());

    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());
    let result = localizer.lookup(&Locale::fr_fr(), &MessageId::tool_mode_draw());
    assert!(result.is_ok());
    assert_eq!(result.expect("Should have found message"), "Draw");
}

#[test]
fn localizer_lookup_returns_error_for_missing_message() {
    let localizer = Localizer::new();
    let result = localizer.lookup(&Locale::en_gb(), &MessageId::from("nonexistent"));
    assert!(result.is_err());
    match result.expect_err("Should have returned error") {
        I18nError::MessageNotFound { message_id, .. } => {
            assert_eq!(message_id, "nonexistent");
        }
        I18nError::UnsupportedLocale { .. } => {
            panic!("Expected MessageNotFound error")
        }
    }
}

#[rstest]
fn localizer_add_catalog_works(fr_test_catalog: Catalog) {
    let mut localizer = Localizer::new();
    localizer.add_catalog(Locale::fr_fr(), fr_test_catalog);
    let result = localizer.lookup(&Locale::fr_fr(), &MessageId::from("test"));
    assert!(result.is_ok());
    assert_eq!(result.expect("Should have found message"), "test_fr");
}

fn default_catalog() -> Catalog {
    Catalog::default_en_gb()
}

#[test]
fn catalog_default_en_gb_contains_chrome_messages() {
    let catalog = default_catalog();
    assert_eq!(catalog.get(&MessageId::chrome_file_new()), Some("New"));
}

#[test]
fn catalog_default_en_gb_contains_tool_tooltip_messages() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.get(&MessageId::tool_tooltip_draw_path()),
        Some("Draw Path")
    );
}

#[test]
fn catalog_default_en_gb_contains_status_messages() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.get(&MessageId::status_saved()),
        Some("Saved: {path}")
    );
}

#[test]
fn catalog_default_en_gb_contains_align_messages() {
    let catalog = default_catalog();
    assert_eq!(catalog.get(&MessageId::align_left()), Some("Align Left"));
}

#[test]
fn catalog_default_en_gb_contains_style_messages() {
    let catalog = default_catalog();
    assert_eq!(catalog.get(&MessageId::style_stroke()), Some("Stroke"));
}

#[test]
fn catalog_default_en_gb_contains_doc_messages() {
    let catalog = default_catalog();
    assert_eq!(catalog.get(&MessageId::doc_untitled()), Some("untitled"));
}

#[test]
fn catalog_default_en_gb_contains_a11y_canvas_message() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.get(&MessageId::a11y_canvas()),
        Some("Drawing canvas")
    );
}

#[test]
fn catalog_default_en_gb_contains_a11y_shape_item_message() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.get(&MessageId::a11y_shape_item()),
        Some("Shape {index}")
    );
}

#[test]
fn catalog_default_en_gb_contains_zoom_ratio_message() {
    let catalog = default_catalog();
    assert_eq!(
        catalog.get(&MessageId::status_zoom_ratio_1_1()),
        Some("1:1")
    );
}

#[test]
fn catalog_default_en_gb_preserves_template_placeholders() {
    let catalog = Catalog::default_en_gb();

    let status_saved = catalog
        .get(&MessageId::status_saved())
        .expect("status_saved should be present in default_en_gb");
    assert_eq!(status_saved, "Saved: {path}");
    assert!(
        status_saved.contains("{path}"),
        "status_saved template should contain {{path}} placeholder"
    );

    let a11y_shape_item = catalog
        .get(&MessageId::a11y_shape_item())
        .expect("a11y_shape_item should be present in default_en_gb");
    assert_eq!(a11y_shape_item, "Shape {index}");
    assert!(
        a11y_shape_item.contains("{index}"),
        "a11y_shape_item template should contain {{index}} placeholder"
    );
}
