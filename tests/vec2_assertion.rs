//! Regression tests for vector assertion test support.

#[path = "common/vec2_assertion.rs"]
mod vec2_assertion;

use gauss::model::Vec2;
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
