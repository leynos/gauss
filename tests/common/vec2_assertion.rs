//! Vector comparison assertions for GPUI integration tests.
use gauss::model::Vec2;
use test_support::{TestSupportError, TestSupportResult};

pub fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) -> TestSupportResult<()> {
    let diff = actual.sub(expected);
    if diff.distance_squared(Vec2::ZERO) > 0.0001 {
        return Err(TestSupportError::expectation(format!(
            "{context}: expected={expected:?} got={actual:?}"
        )));
    }
    Ok(())
}
