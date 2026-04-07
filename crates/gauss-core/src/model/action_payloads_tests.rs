//! Tests for `action_payloads` module.

use std::hash::{Hash, Hasher};

use super::*;

/// Helper for asserting that `normalize_float` produces the expected result.
#[expect(
    clippy::float_cmp,
    reason = "exact float equality is intentional in tests for -0.0 normalization"
)]
fn assert_normalizes(input: f32, expected: f32) {
    assert_eq!(
        normalize_float(input),
        expected,
        "normalize_float({input}) must be {expected}"
    );
}

/// Helper for asserting that `StrokeWidth::new` rejects invalid values.
fn assert_stroke_width_rejected(points: f32) {
    assert!(
        StrokeWidth::new(Points(points)).is_none(),
        "StrokeWidth::new(Points({points})) must be rejected"
    );
}

/// Helper for asserting that `StrokeWidth::new` accepts valid values.
fn assert_stroke_width_accepted(points: f32) {
    assert!(
        StrokeWidth::new(Points(points)).is_some(),
        "StrokeWidth::new(Points({points})) must be accepted"
    );
}

/// Helper for asserting that `Position::new` rejects invalid points.
fn assert_position_rejected(x: f32, y: f32) {
    assert!(
        Position::new(Point { x, y }).is_none(),
        "Position::new(Point {{ x: {x}, y: {y} }}) must be rejected"
    );
}

/// Helper for asserting that `Position::new` accepts valid points.
fn assert_position_accepted(x: f32, y: f32) {
    assert!(
        Position::new(Point { x, y }).is_some(),
        "Position::new(Point {{ x: {x}, y: {y} }}) must be accepted"
    );
}

/// Helper for asserting that `Size::new` rejects invalid dimensions.
fn assert_size_rejected(width: f32, height: f32) {
    assert!(
        Size::new(Dimensions { width, height }).is_none(),
        "Size::new(Dimensions {{ width: {width}, height: {height} }}) must be rejected"
    );
}

/// Helper for asserting that `Size::new` accepts valid dimensions.
fn assert_size_accepted(width: f32, height: f32) {
    assert!(
        Size::new(Dimensions { width, height }).is_some(),
        "Size::new(Dimensions {{ width: {width}, height: {height} }}) must be accepted"
    );
}

#[test]
fn stroke_width_rejects_negative_and_non_finite() {
    // Negative should be rejected
    assert_stroke_width_rejected(-1.0);
    assert_stroke_width_rejected(-0.1);

    // Non-finite should be rejected
    assert_stroke_width_rejected(f32::NAN);
    assert_stroke_width_rejected(f32::INFINITY);
    assert_stroke_width_rejected(f32::NEG_INFINITY);

    // Valid values should be accepted
    assert_stroke_width_accepted(0.0);
    assert_stroke_width_accepted(1.0);
    assert_stroke_width_accepted(2.5);
}

#[test]
fn stroke_width_normalizes_negative_zero() {
    let sw_neg = StrokeWidth::new(Points(-0.0)).expect("StrokeWidth::new should accept -0.0");
    let sw_pos = StrokeWidth::new(Points(0.0)).expect("StrokeWidth::new should accept 0.0");
    // Both should hash to the same value
    let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
    let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
    sw_neg.hash(&mut hasher_neg);
    sw_pos.hash(&mut hasher_pos);
    assert_eq!(hasher_neg.finish(), hasher_pos.finish());
    // And be equal
    assert_eq!(sw_neg, sw_pos);
}

#[test]
fn unit_f32_rejects_out_of_range() {
    assert!(UnitF32::try_from(-0.1).is_err());
    assert!(UnitF32::try_from(1.1).is_err());
    assert!(UnitF32::try_from(2.0).is_err());
}

#[test]
fn unit_f32_rejects_non_finite() {
    assert!(UnitF32::try_from(f32::NAN).is_err());
    assert!(UnitF32::try_from(f32::INFINITY).is_err());
    assert!(UnitF32::try_from(f32::NEG_INFINITY).is_err());
}

#[test]
fn unit_f32_accepts_valid_values() {
    assert!(UnitF32::try_from(0.0).is_ok());
    assert!(UnitF32::try_from(1.0).is_ok());
    assert!(UnitF32::try_from(0.5).is_ok());
}

#[test]
fn opacity_rejects_non_finite() {
    // UnitF32::try_from rejects non-finite values
    assert!(UnitF32::try_from(f32::NAN).is_err());
    assert!(UnitF32::try_from(f32::INFINITY).is_err());
    assert!(UnitF32::try_from(f32::NEG_INFINITY).is_err());

    // Valid values should be accepted through the full chain
    assert!(
        Opacity::new(UnitF32::try_from(0.5).expect("UnitF32::try_from should accept 0.5"))
            .is_some()
    );
}

#[test]
fn opacity_normalizes_negative_zero() {
    let op_neg =
        Opacity::new(UnitF32::try_from(-0.0).expect("UnitF32::try_from should accept -0.0"))
            .expect("Opacity::new should accept valid UnitF32");
    let op_pos = Opacity::new(UnitF32::try_from(0.0).expect("UnitF32::try_from should accept 0.0"))
        .expect("Opacity::new should accept valid UnitF32");
    // Both should hash to the same value
    let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
    let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
    op_neg.hash(&mut hasher_neg);
    op_pos.hash(&mut hasher_pos);
    assert_eq!(hasher_neg.finish(), hasher_pos.finish());
    // And be equal
    assert_eq!(op_neg, op_pos);
}

#[test]
fn position_rejects_non_finite() {
    // Non-finite should be rejected
    assert_position_rejected(f32::NAN, 0.0);
    assert_position_rejected(0.0, f32::NAN);
    assert_position_rejected(f32::INFINITY, 0.0);
    assert_position_rejected(0.0, f32::NEG_INFINITY);

    // Valid values should be accepted
    assert_position_accepted(0.0, 0.0);
    assert_position_accepted(1.0, 2.0);
}

#[test]
fn position_normalizes_negative_zero() {
    let pos_neg = Position::new(Point { x: -0.0, y: -0.0 })
        .expect("Position::new should accept negative zero coordinates");
    let pos_pos = Position::new(Point { x: 0.0, y: 0.0 })
        .expect("Position::new should accept zero coordinates");
    // Both should hash to the same value
    let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
    let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
    pos_neg.hash(&mut hasher_neg);
    pos_pos.hash(&mut hasher_pos);
    assert_eq!(hasher_neg.finish(), hasher_pos.finish());
    // And be equal
    assert_eq!(pos_neg, pos_pos);
}

#[test]
fn size_rejects_negative_and_non_finite() {
    // Negative dimensions should be rejected
    assert_size_rejected(-1.0, 1.0);
    assert_size_rejected(1.0, -1.0);
    assert_size_rejected(-0.1, -0.1);

    // Non-finite should be rejected
    assert_size_rejected(f32::NAN, 1.0);
    assert_size_rejected(1.0, f32::INFINITY);
    assert_size_rejected(f32::NEG_INFINITY, 1.0);

    // Valid values should be accepted
    assert_size_accepted(0.0, 0.0);
    assert_size_accepted(100.0, 200.0);
}

#[test]
fn size_normalizes_negative_zero() {
    let size_neg = Size::new(Dimensions {
        width: -0.0,
        height: -0.0,
    })
    .expect("Size::new should accept negative zero dimensions");
    let size_pos = Size::new(Dimensions {
        width: 0.0,
        height: 0.0,
    })
    .expect("Size::new should accept zero dimensions");
    // Both should hash to the same value
    let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
    let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
    size_neg.hash(&mut hasher_neg);
    size_pos.hash(&mut hasher_pos);
    assert_eq!(hasher_neg.finish(), hasher_pos.finish());
    // And be equal
    assert_eq!(size_neg, size_pos);
}

#[test]
fn rotation_rejects_non_finite() {
    // Non-finite should be rejected
    assert!(Rotation::new(Degrees(f32::NAN)).is_none());
    assert!(Rotation::new(Degrees(f32::INFINITY)).is_none());
    assert!(Rotation::new(Degrees(f32::NEG_INFINITY)).is_none());

    // Valid values should be accepted
    assert!(Rotation::new(Degrees(0.0)).is_some());
    assert!(Rotation::new(Degrees(45.0)).is_some());
    assert!(Rotation::new(Degrees(-90.0)).is_some());
}

#[test]
fn rotation_normalizes_negative_zero() {
    let rot_neg = Rotation::new(Degrees(-0.0)).expect("Rotation::new should accept -0.0 degrees");
    let rot_pos = Rotation::new(Degrees(0.0)).expect("Rotation::new should accept 0.0 degrees");
    // Both should hash to the same value
    let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
    let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
    rot_neg.hash(&mut hasher_neg);
    rot_pos.hash(&mut hasher_pos);
    assert_eq!(hasher_neg.finish(), hasher_pos.finish());
    // And be equal
    assert_eq!(rot_neg, rot_pos);
}

#[test]
fn normalize_float_converts_negative_zero() {
    assert_normalizes(-0.0, 0.0);
    assert_normalizes(0.0, 0.0);
    assert_normalizes(1.0, 1.0);
    assert_normalizes(-1.0, -1.0);
}
