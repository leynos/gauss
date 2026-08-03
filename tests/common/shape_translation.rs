//! Shape-translation assertions for GPUI integration tests.

use gauss::model::{Shape, Vec2};
use test_support::{TestSupportError, TestSupportResult};

/// Checks that every anchor in `shape` equals its original plus `delta`.
///
/// Both shapes must have the same anchor count. Each translated anchor may
/// differ from its expected document-space position by at most `0.01` units.
///
/// # Errors
///
/// Returns a test-support expectation error when anchor counts differ or an
/// anchor exceeds the translation tolerance; diagnostics include `context`.
pub fn assert_shape_translated_by_delta(
    shape: &Shape,
    original: &Shape,
    delta: Vec2,
    context: &str,
) -> TestSupportResult<()> {
    if shape.path.anchors.len() != original.path.anchors.len() {
        return Err(TestSupportError::expectation(format!(
            "anchor count mismatch: {context}"
        )));
    }

    for (current, start) in shape.path.anchors.iter().zip(original.path.anchors.iter()) {
        let expected = start.pos.add(delta);
        let diff = current.pos.sub(expected);
        if diff.distance_squared(Vec2::ZERO) > 0.0001 {
            return Err(TestSupportError::expectation(format!(
                "anchor did not move by expected delta: {context}; start={:?} expected={:?} got={:?} delta={:?}",
                start.pos, expected, current.pos, delta
            )));
        }
    }
    Ok(())
}
