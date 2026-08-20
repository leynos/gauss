//! Regression tests for vector assertion test support.

#[path = "common/vec2_assertion.rs"]
mod vec2_assertion;

use gauss::model::Vec2;
use proptest::prelude::*;
use test_support::TestSupportError;
use vec2_assertion::assert_vec2_close;

#[test]
fn rejects_vector_with_nan_component() {
    let error = assert_vec2_close(Vec2::new(f32::NAN, 0.0), Vec2::ZERO, "non-finite vector")
        .expect_err("a NaN component should fail the vector comparison");

    assert!(
        matches!(error, TestSupportError::Expectation { .. }),
        "a non-finite vector should return an expectation error"
    );
}

proptest! {
    #[test]
    fn accepts_nearby_finite_vectors(
        x in -1_000.0f32..1_000.0,
        y in -1_000.0f32..1_000.0,
        x_delta in -0.005f32..0.005,
        y_delta in -0.005f32..0.005,
    ) {
        let expected = Vec2::new(x, y);
        let actual = expected.add(Vec2::new(x_delta, y_delta));

        prop_assert!(assert_vec2_close(actual, expected, "finite vectors").is_ok());
    }

    #[test]
    fn rejects_non_finite_components_in_either_vector(
        x in -1_000.0f32..1_000.0,
        y in -1_000.0f32..1_000.0,
        non_finite in prop_oneof![
            Just(f32::NAN),
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
        ],
        position in 0u8..4,
    ) {
        let finite = Vec2::new(x, y);
        let (actual, expected) = match position {
            0 => (Vec2::new(non_finite, y), finite),
            1 => (Vec2::new(x, non_finite), finite),
            2 => (finite, Vec2::new(non_finite, y)),
            _ => (finite, Vec2::new(x, non_finite)),
        };

        let is_expectation_error = matches!(
            assert_vec2_close(actual, expected, "non-finite vectors"),
            Err(TestSupportError::Expectation { .. })
        );

        prop_assert!(is_expectation_error);
    }
}
