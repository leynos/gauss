//! Tests for `ResourceStore` behaviour.

use rstest::{fixture, rstest};

use crate::model::{
    Gradient, GradientKind, GradientStop, LinearGradient, PatternResource, ResourceStore, Rgba,
    SymbolResource, Vec2,
};

#[fixture]
fn sample_linear_gradient() -> impl Fn(&str) -> Gradient {
    |svg_id| {
        Gradient::new(
            svg_id,
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 1.0),
                vec![
                    GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                    GradientStop::new(1.0, Rgba::new(0, 0, 255, 255)),
                ],
            )),
        )
    }
}

#[rstest]
fn new_resource_store_is_empty() {
    let store = ResourceStore::new();
    assert!(store.is_empty());
    assert_eq!(store.gradient_count(), 0);
    assert_eq!(store.pattern_count(), 0);
    assert_eq!(store.symbol_count(), 0);
}

#[rstest]
fn gradient_svg_ids_are_unique(sample_linear_gradient: impl Fn(&str) -> Gradient) {
    let mut store = ResourceStore::new();
    let first = store.insert_gradient(sample_linear_gradient("brand"));
    let second = store.insert_gradient(sample_linear_gradient("brand"));

    assert_ne!(first, second);
    assert_eq!(
        store
            .gradient(first)
            .map(|gradient| gradient.svg_id.as_str()),
        Some("brand")
    );
    assert_eq!(
        store
            .gradient(second)
            .map(|gradient| gradient.svg_id.as_str()),
        Some("brand-1")
    );
}

#[rstest]
fn can_lookup_and_remove_pattern_by_id() {
    let mut store = ResourceStore::new();
    let id = store.insert_pattern(PatternResource::new("dots", "<circle />"));

    assert_eq!(store.pattern_id_for_svg_id("dots"), Some(id));
    assert!(store.remove_pattern(id).is_some());
    assert!(store.pattern(id).is_none());
    assert!(store.pattern_id_for_svg_id("dots").is_none());
}

#[rstest]
fn symbol_svg_ids_are_unique() {
    let mut store = ResourceStore::new();
    let first = store.insert_symbol(SymbolResource::new("badge", None, "<rect />"));
    let second = store.insert_symbol(SymbolResource::new("badge", None, "<rect />"));

    assert_ne!(first, second);
    assert_eq!(
        store.symbol(first).map(|symbol| symbol.svg_id.as_str()),
        Some("badge")
    );
    assert_eq!(
        store.symbol(second).map(|symbol| symbol.svg_id.as_str()),
        Some("badge-1")
    );
}

#[rstest]
fn blank_svg_ids_fall_back_to_resource_prefixes(sample_linear_gradient: impl Fn(&str) -> Gradient) {
    let mut store = ResourceStore::new();

    let gradient_first = store.insert_gradient(sample_linear_gradient("   "));
    let gradient_second = store.insert_gradient(sample_linear_gradient(""));
    let pattern_first = store.insert_pattern(PatternResource::new(" ", "<circle />"));
    let pattern_second = store.insert_pattern(PatternResource::new("", "<circle />"));
    let symbol_first = store.insert_symbol(SymbolResource::new("   ", None, "<rect />"));
    let symbol_second = store.insert_symbol(SymbolResource::new("", None, "<rect />"));

    assert_eq!(
        store
            .gradient(gradient_first)
            .map(|gradient| gradient.svg_id.as_str()),
        Some("gradient")
    );
    assert_eq!(
        store
            .gradient(gradient_second)
            .map(|gradient| gradient.svg_id.as_str()),
        Some("gradient-1")
    );
    assert_eq!(
        store
            .pattern(pattern_first)
            .map(|pattern| pattern.svg_id.as_str()),
        Some("pattern")
    );
    assert_eq!(
        store
            .pattern(pattern_second)
            .map(|pattern| pattern.svg_id.as_str()),
        Some("pattern-1")
    );
    assert_eq!(
        store
            .symbol(symbol_first)
            .map(|symbol| symbol.svg_id.as_str()),
        Some("symbol")
    );
    assert_eq!(
        store
            .symbol(symbol_second)
            .map(|symbol| symbol.svg_id.as_str()),
        Some("symbol-1")
    );
}
