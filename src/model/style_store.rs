//! Style store for named and reusable styles.
//!
//! This module provides storage for named style definitions that can be applied
//! to shapes. Styles capture appearance attributes like fill, stroke, opacity,
//! and effects.
//!
//! This is a stub preparing for roadmap task 0.2.3 (Define resource stores).
//! The full implementation will support:
//!
//! - Named styles (user-defined appearance presets)
//! - Style inheritance (styles can extend other styles)
//! - Style linking (shapes reference style by ID, updates propagate)

/// Storage for named style definitions.
///
/// Named styles allow users to create reusable appearance presets. When a
/// linked style is updated, all shapes using that style update automatically.
///
/// # Future Expansion
///
/// This struct will grow to include:
///
/// - `styles: HashMap<StyleId, NamedStyle>`
/// - `default_style: StyleId`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleStore {
    /// Placeholder field to prevent zero-sized type.
    ///
    /// This will be replaced with actual style storage in task 0.2.3.
    _placeholder: (),
}

impl StyleStore {
    /// Construct an empty style store.
    #[must_use]
    pub const fn new() -> Self {
        Self { _placeholder: () }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn new_style_store_is_empty() {
        let store = StyleStore::new();
        // Stub has no contents to verify; just ensure construction works
        assert_eq!(store, StyleStore::default());
    }

    #[rstest]
    fn style_store_is_clone() {
        let store = StyleStore::new();
        let cloned = store.clone();
        assert_eq!(store, cloned);
    }
}
