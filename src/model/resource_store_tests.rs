//! Tests for `ResourceStore` behaviour.

use rstest::rstest;

use crate::model::{
    Gradient, GradientKind, GradientStop, LinearGradient, PatternResource, ResourceStore, Rgba,
    Vec2,
};

fn sample_linear_gradient(svg_id: &str) -> Gradient {
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

#[rstest]
fn new_resource_store_is_empty() {
    let store = ResourceStore::new();
    assert!(store.is_empty());
    assert_eq!(store.gradient_count(), 0);
    assert_eq!(store.pattern_count(), 0);
    assert_eq!(store.symbol_count(), 0);
}

#[rstest]
fn gradient_svg_ids_are_unique() {
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
