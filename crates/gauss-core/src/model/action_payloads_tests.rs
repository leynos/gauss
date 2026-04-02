//! Tests for `action_payloads` module.
#![expect(
    clippy::unwrap_used,
    reason = "unwrap is appropriate in tests for concise assertions"
)]
#![expect(
    clippy::float_cmp,
    reason = "exact float equality is intentional in tests for -0.0 normalization"
)]

use std::hash::{Hash, Hasher};

use super::*;

#[test]
fn stroke_width_rejects_negative_and_non_finite() {
    // Negative should be rejected
    assert!(StrokeWidth::new(Points(-1.0)).is_none());
    assert!(StrokeWidth::new(Points(-0.1)).is_none());

    // Non-finite should be rejected
    assert!(StrokeWidth::new(Points(f32::NAN)).is_none());
    assert!(StrokeWidth::new(Points(f32::INFINITY)).is_none());
    assert!(StrokeWidth::new(Points(f32::NEG_INFINITY)).is_none());

    // Valid values should be accepted
    assert!(StrokeWidth::new(Points(0.0)).is_some());
    assert!(StrokeWidth::new(Points(1.0)).is_some());
    assert!(StrokeWidth::new(Points(2.5)).is_some());
}

#[test]
fn stroke_width_normalizes_negative_zero() {
    let sw_neg = StrokeWidth::new(Points(-0.0)).unwrap();
    let sw_pos = StrokeWidth::new(Points(0.0)).unwrap();
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
    assert!(Opacity::new(UnitF32::try_from(0.5).unwrap()).is_some());
}

#[test]
fn opacity_normalizes_negative_zero() {
    let op_neg = Opacity::new(UnitF32::try_from(-0.0).unwrap()).unwrap();
    let op_pos = Opacity::new(UnitF32::try_from(0.0).unwrap()).unwrap();
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
    assert!(
        Position::new(Point {
            x: f32::NAN,
            y: 0.0
        })
        .is_none()
    );
    assert!(
        Position::new(Point {
            x: 0.0,
            y: f32::NAN
        })
        .is_none()
    );
    assert!(
        Position::new(Point {
            x: f32::INFINITY,
            y: 0.0
        })
        .is_none()
    );
    assert!(
        Position::new(Point {
            x: 0.0,
            y: f32::NEG_INFINITY
        })
        .is_none()
    );

    // Valid values should be accepted
    assert!(Position::new(Point { x: 0.0, y: 0.0 }).is_some());
    assert!(Position::new(Point { x: 1.0, y: 2.0 }).is_some());
}

#[test]
fn position_normalizes_negative_zero() {
    let pos_neg = Position::new(Point { x: -0.0, y: -0.0 }).unwrap();
    let pos_pos = Position::new(Point { x: 0.0, y: 0.0 }).unwrap();
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
    assert!(
        Size::new(Dimensions {
            width: -1.0,
            height: 1.0
        })
        .is_none()
    );
    assert!(
        Size::new(Dimensions {
            width: 1.0,
            height: -1.0
        })
        .is_none()
    );
    assert!(
        Size::new(Dimensions {
            width: -0.1,
            height: -0.1
        })
        .is_none()
    );

    // Non-finite should be rejected
    assert!(
        Size::new(Dimensions {
            width: f32::NAN,
            height: 1.0
        })
        .is_none()
    );
    assert!(
        Size::new(Dimensions {
            width: 1.0,
            height: f32::INFINITY
        })
        .is_none()
    );
    assert!(
        Size::new(Dimensions {
            width: f32::NEG_INFINITY,
            height: 1.0
        })
        .is_none()
    );

    // Valid values should be accepted
    assert!(
        Size::new(Dimensions {
            width: 0.0,
            height: 0.0
        })
        .is_some()
    );
    assert!(
        Size::new(Dimensions {
            width: 100.0,
            height: 200.0
        })
        .is_some()
    );
}

#[test]
fn size_normalizes_negative_zero() {
    let size_neg = Size::new(Dimensions {
        width: -0.0,
        height: -0.0,
    })
    .unwrap();
    let size_pos = Size::new(Dimensions {
        width: 0.0,
        height: 0.0,
    })
    .unwrap();
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
    let rot_neg = Rotation::new(Degrees(-0.0)).unwrap();
    let rot_pos = Rotation::new(Degrees(0.0)).unwrap();
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
    assert_eq!(normalize_float(-0.0), 0.0);
    assert_eq!(normalize_float(0.0), 0.0);
    assert_eq!(normalize_float(1.0), 1.0);
    assert_eq!(normalize_float(-1.0), -1.0);
}
