//! Localization catalog fixtures for GPUI integration tests.

/// Builds and returns an owned catalogue from borrowed message key-value pairs.
pub fn make_test_catalog(entries: &[(&str, &str)]) -> gauss::i18n::Catalog {
    use std::collections::HashMap;

    let mut messages = HashMap::new();
    for (key, value) in entries {
        messages.insert((*key).to_owned(), (*value).to_owned());
    }
    gauss::i18n::Catalog::from_messages(messages)
}

/// Returns an owned French catalogue for the test tool and edge-mode messages.
pub fn test_french_catalog() -> gauss::i18n::Catalog {
    make_test_catalog(&[
        ("tool_mode.draw", "Dessiner"),
        ("tool_mode.manipulate", "Manipuler"),
        ("edge_mode.line", "Ligne"),
        ("edge_mode.bezier_auto", "Bézier (auto)"),
    ])
}
