//! Vector comparison assertions for GPUI integration tests.
use gauss::model::Vec2;
use test_support::{TestSupportError, TestSupportResult};

/// Checks that two vectors differ by at most `0.01` Euclidean units.
///
/// # Errors
///
/// Returns an expectation error containing `context` and both vectors when the
/// squared distance exceeds `0.0001`.
pub fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) -> TestSupportResult<()> {
    let diff = actual.sub(expected);
    let squared_distance = diff.distance_squared(Vec2::ZERO);
    if !squared_distance.is_finite() || squared_distance > 0.0001 {
        return Err(TestSupportError::expectation(format!(
            "{context}: expected={expected:?} got={actual:?}"
        )));
    }
    Ok(())
}
