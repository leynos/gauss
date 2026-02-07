//! Style store for named and reusable styles.
//!
//! This module provides storage for named style definitions that can be applied
//! to shapes. Styles capture appearance attributes like fill, stroke, opacity,
//! and effects.
//!
//! 0.2.3 introduces a concrete style registry keyed by stable style IDs.
//! The store keeps a name index for deterministic lookups and unique naming.

use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type};

use crate::model::PaintStyle;

new_key_type! {
    /// Identifier for a named style in [`StyleStore`].
    pub struct StyleId;
}

/// A named reusable style definition.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedStyle {
    /// Human-readable style name.
    pub name: String,
    /// Style payload applied to shapes.
    pub style: PaintStyle,
}

impl NamedStyle {
    /// Construct a named style definition.
    #[must_use]
    pub fn new(name: impl Into<String>, style: PaintStyle) -> Self {
        Self {
            name: name.into(),
            style,
        }
    }
}

/// Storage for named style definitions.
#[derive(Clone, Debug, Default)]
pub struct StyleStore {
    styles: SlotMap<StyleId, NamedStyle>,
    names: HashMap<String, StyleId>,
    default_style: Option<StyleId>,
}

impl PartialEq for StyleStore {
    fn eq(&self, other: &Self) -> bool {
        if self.styles.len() != other.styles.len() {
            return false;
        }

        for (id, style) in &self.styles {
            if other.styles.get(id) != Some(style) {
                return false;
            }
        }

        self.names == other.names && self.default_style == other.default_style
    }
}

impl StyleStore {
    /// Construct an empty style store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether no styles exist.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    /// Return the number of stored styles.
    #[must_use]
    pub fn style_count(&self) -> usize {
        self.styles.len()
    }

    /// Return the default style identifier, if configured.
    #[must_use]
    pub const fn default_style(&self) -> Option<StyleId> {
        self.default_style
    }

    /// Set the default style. Returns `true` when the style exists.
    pub fn set_default_style(&mut self, id: StyleId) -> bool {
        if self.styles.contains_key(id) {
            self.default_style = Some(id);
            true
        } else {
            false
        }
    }

    /// Insert a style and return its assigned identifier.
    pub fn insert(&mut self, mut style: NamedStyle) -> StyleId {
        style.name = make_unique_name(&style.name, &self.names);
        let name = style.name.clone();
        let id = self.styles.insert(style);
        self.names.insert(name, id);
        id
    }

    /// Return a style by identifier.
    #[must_use]
    pub fn style(&self, id: StyleId) -> Option<&NamedStyle> {
        self.styles.get(id)
    }

    /// Return a mutable style by identifier.
    #[must_use]
    pub fn style_mut(&mut self, id: StyleId) -> Option<&mut NamedStyle> {
        self.styles.get_mut(id)
    }

    /// Return a style identifier for a given name.
    #[must_use]
    pub fn style_id_for_name(&self, name: &str) -> Option<StyleId> {
        self.names.get(name).copied()
    }

    /// Rename a style while preserving uniqueness.
    pub fn rename(&mut self, id: StyleId, requested_name: &str) -> bool {
        let Some(existing) = self.styles.get(id) else {
            return false;
        };

        let old_name = existing.name.clone();
        let mut candidate_names = self.names.clone();
        candidate_names.remove(old_name.as_str());
        let next_name = make_unique_name(requested_name, &candidate_names);

        let Some(style) = self.styles.get_mut(id) else {
            return false;
        };
        style.name.clone_from(&next_name);
        self.names.remove(old_name.as_str());
        self.names.insert(next_name, id);
        true
    }

    /// Remove a style by identifier.
    pub fn remove(&mut self, id: StyleId) -> Option<NamedStyle> {
        let removed = self.styles.remove(id)?;
        self.names.remove(removed.name.as_str());
        if self.default_style == Some(id) {
            self.default_style = None;
        }
        Some(removed)
    }

    /// Iterate stored styles as `(id, style)` tuples.
    pub fn iter(&self) -> impl Iterator<Item = (StyleId, &NamedStyle)> + '_ {
        self.styles.iter()
    }
}

fn make_unique_name<T>(requested: &str, existing: &HashMap<String, T>) -> String {
    let trimmed = requested.trim();
    let base = if trimmed.is_empty() {
        "Style".to_owned()
    } else {
        trimmed.to_owned()
    };

    if !existing.contains_key(base.as_str()) {
        return base;
    }

    let mut suffix = 1_u32;
    loop {
        let candidate = format!("{base} {suffix}");
        if !existing.contains_key(candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    //! Tests for style storage behaviour.

    use super::*;
    use crate::model::Rgba;
    use rstest::rstest;

    fn sample_style(name: &str, width: f32) -> NamedStyle {
        NamedStyle::new(
            name,
            PaintStyle::new(Some(Rgba::new(255, 0, 0, 255)), width, None),
        )
    }

    #[rstest]
    fn new_style_store_is_empty() {
        let store = StyleStore::new();
        assert!(store.is_empty());
        assert_eq!(store.style_count(), 0);
        assert_eq!(store.default_style(), None);
    }

    #[rstest]
    fn style_names_are_unique() {
        let mut store = StyleStore::new();
        let first = store.insert(sample_style("Primary", 1.0));
        let second = store.insert(sample_style("Primary", 2.0));

        assert_ne!(first, second);
        assert_eq!(
            store.style(first).map(|style| style.name.as_str()),
            Some("Primary")
        );
        assert_eq!(
            store.style(second).map(|style| style.name.as_str()),
            Some("Primary 1")
        );
    }

    #[rstest]
    fn rename_preserves_uniqueness() {
        let mut store = StyleStore::new();
        let first = store.insert(sample_style("Main", 1.0));
        let _second = store.insert(sample_style("Accent", 2.0));

        assert!(store.rename(first, "Accent"));
        assert_eq!(
            store.style(first).map(|style| style.name.as_str()),
            Some("Accent 1")
        );
    }

    #[rstest]
    fn removing_default_style_clears_default_marker() {
        let mut store = StyleStore::new();
        let id = store.insert(sample_style("Default", 1.0));
        assert!(store.set_default_style(id));
        assert_eq!(store.default_style(), Some(id));

        let _removed = store.remove(id);
        assert_eq!(store.default_style(), None);
    }
}
