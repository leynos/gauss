//! Resource store for shared document resources.
//!
//! This module provides storage for reusable resources like gradients, patterns,
//! symbols, and markers. Resources are referenced by ID from style definitions.
//!
//! This is a stub preparing for roadmap task 0.2.3 (Define resource stores).
//! The full implementation will support:
//!
//! - Gradient definitions (linear, radial)
//! - Pattern definitions (tile-based fills)
//! - Symbol definitions (reusable artwork)
//! - Marker definitions (arrowheads, line caps)

/// Storage for shared document resources.
///
/// Resources are shared definitions that can be referenced by multiple shapes.
/// For example, a gradient can be defined once and applied to many shapes'
/// fill or stroke.
///
/// # Future Expansion
///
/// This struct will grow to include:
///
/// - `gradients: HashMap<GradientId, Gradient>`
/// - `patterns: HashMap<PatternId, Pattern>`
/// - `symbols: HashMap<SymbolId, Symbol>`
/// - `markers: HashMap<MarkerId, Marker>`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceStore {
    /// Placeholder field to prevent zero-sized type.
    ///
    /// This will be replaced with actual resource storage in task 0.2.3.
    _placeholder: (),
}

impl ResourceStore {
    /// Construct an empty resource store.
    #[must_use]
    pub const fn new() -> Self {
        Self { _placeholder: () }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for resource store defaults.

    use super::*;

    #[test]
    fn new_resource_store_is_empty() {
        let store = ResourceStore::new();
        // Stub has no contents to verify; just ensure construction works
        assert_eq!(store, ResourceStore::default());
    }

    #[test]
    fn resource_store_is_clone() {
        let store = ResourceStore::new();
        let cloned = store.clone();
        assert_eq!(store, cloned);
    }
}
