//! Tests for `action_payloads` module.

use std::hash::{Hash, Hasher};

use rstest::rstest;

use super::*;

/// Macro for asserting that `normalize_float` produces the expected result.
macro_rules! assert_normalizes {
    ($input:expr, $expected:expr) => {
        assert_eq!(
            normalize_float($input),
            $expected,
            "normalize_float({}) must be {}",
            $input,
            $expected
        );
    };
}

/// Asserts hash and value equality of two constructed instances.
fn assert_neg_zero_equal<T: Hash + Eq + std::fmt::Debug>(a: &T, b: &T, ctx: &str) {
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(
        h1.finish(),
        h2.finish(),
        "hash must be equal for -0.0 vs +0.0 ({ctx})"
    );
    assert_eq!(a, b, "value must be equal for -0.0 vs +0.0 ({ctx})");
}

#[rstest]
#[case(-1.0)]
#[case(-0.1)]
#[case(f32::NAN)]
#[case(f32::INFINITY)]
#[case(f32::NEG_INFINITY)]
fn stroke_width_rejects_invalid(#[case] value: f32) {
    assert!(
        StrokeWidth::new(Points(value)).is_none(),
        "StrokeWidth::new(Points({value})) must be rejected"
    );
}

#[rstest]
#[case(0.0)]
#[case(1.0)]
#[case(2.5)]
fn stroke_width_accepts_valid(#[case] value: f32) {
    assert!(
        StrokeWidth::new(Points(value)).is_some(),
        "StrokeWidth::new(Points({value})) must be accepted"
    );
}

#[test]
fn stroke_width_normalizes_negative_zero() {
    assert_neg_zero_equal(
        &StrokeWidth::new(Points(-0.0)).expect("StrokeWidth::new should accept -0.0"),
        &StrokeWidth::new(Points(0.0)).expect("StrokeWidth::new should accept 0.0"),
        "StrokeWidth",
    );
}

#[rstest]
#[case(-0.1)]
#[case(1.1)]
#[case(2.0)]
#[case(f32::NAN)]
#[case(f32::INFINITY)]
#[case(f32::NEG_INFINITY)]
fn unit_f32_rejects_invalid(#[case] value: f32) {
    assert!(
        UnitF32::try_from(value).is_err(),
        "UnitF32::try_from({value}) must fail"
    );
}

#[rstest]
#[case(0.0)]
#[case(1.0)]
#[case(0.5)]
fn unit_f32_accepts_valid(#[case] value: f32) {
    assert!(
        UnitF32::try_from(value).is_ok(),
        "UnitF32::try_from({value}) must succeed"
    );
}

#[rstest]
#[case(f32::NAN)]
#[case(f32::INFINITY)]
#[case(f32::NEG_INFINITY)]
fn opacity_rejects_non_finite(#[case] value: f32) {
    assert!(
        UnitF32::try_from(value).is_err(),
        "UnitF32::try_from({value}) must fail"
    );
}

#[test]
fn opacity_accepts_valid() {
    let unit = UnitF32::try_from(0.5).expect("UnitF32::try_from should accept 0.5");
    assert!(
        Opacity::new(unit).is_some(),
        "Opacity::new should accept valid UnitF32"
    );
}

#[test]
fn opacity_normalizes_negative_zero() {
    let neg = Opacity::new(UnitF32::try_from(-0.0).expect("valid"))
        .expect("Opacity::new should accept valid UnitF32");
    let pos = Opacity::new(UnitF32::try_from(0.0).expect("valid"))
        .expect("Opacity::new should accept valid UnitF32");
    assert_neg_zero_equal(&neg, &pos, "Opacity");
}

#[rstest]
#[case(f32::NAN, 0.0)]
#[case(0.0, f32::NAN)]
#[case(f32::INFINITY, 0.0)]
#[case(0.0, f32::NEG_INFINITY)]
fn position_rejects_non_finite(#[case] x: f32, #[case] y: f32) {
    assert!(
        Position::new(Point { x, y }).is_none(),
        "Position::new(Point {{ x: {x}, y: {y} }}) must be rejected"
    );
}

#[rstest]
#[case(0.0, 0.0)]
#[case(1.0, 2.0)]
fn position_accepts_valid(#[case] x: f32, #[case] y: f32) {
    assert!(
        Position::new(Point { x, y }).is_some(),
        "Position::new(Point {{ x: {x}, y: {y} }}) must be accepted"
    );
}

#[test]
fn position_normalizes_negative_zero_x() {
    assert_neg_zero_equal(
        &Position::new(Point { x: -0.0, y: 5.0 }).expect("Position::new should accept -0.0"),
        &Position::new(Point { x: 0.0, y: 5.0 }).expect("Position::new should accept 0.0"),
        "Position(x)",
    );
}

#[test]
fn position_normalizes_negative_zero_y() {
    assert_neg_zero_equal(
        &Position::new(Point { x: 5.0, y: -0.0 }).expect("Position::new should accept -0.0"),
        &Position::new(Point { x: 5.0, y: 0.0 }).expect("Position::new should accept 0.0"),
        "Position(y)",
    );
}

#[rstest]
#[case(-1.0, 1.0)]
#[case(1.0, -1.0)]
#[case(-0.1, -0.1)]
#[case(f32::NAN, 1.0)]
#[case(1.0, f32::INFINITY)]
#[case(f32::NEG_INFINITY, 1.0)]
fn size_rejects_invalid(#[case] width: f32, #[case] height: f32) {
    assert!(
        Size::new(Dimensions { width, height }).is_none(),
        "Size::new(Dimensions {{ width: {width}, height: {height} }}) must be rejected"
    );
}

#[rstest]
#[case(0.0, 0.0)]
#[case(100.0, 200.0)]
fn size_accepts_valid(#[case] width: f32, #[case] height: f32) {
    assert!(
        Size::new(Dimensions { width, height }).is_some(),
        "Size::new(Dimensions {{ width: {width}, height: {height} }}) must be accepted"
    );
}

#[test]
fn size_normalizes_negative_zero_width() {
    assert_neg_zero_equal(
        &Size::new(Dimensions {
            width: -0.0,
            height: 10.0,
        })
        .expect("Size::new should accept -0.0"),
        &Size::new(Dimensions {
            width: 0.0,
            height: 10.0,
        })
        .expect("Size::new should accept 0.0"),
        "Size(width)",
    );
}

#[test]
fn size_normalizes_negative_zero_height() {
    assert_neg_zero_equal(
        &Size::new(Dimensions {
            width: 10.0,
            height: -0.0,
        })
        .expect("Size::new should accept -0.0"),
        &Size::new(Dimensions {
            width: 10.0,
            height: 0.0,
        })
        .expect("Size::new should accept 0.0"),
        "Size(height)",
    );
}

#[rstest]
#[case(f32::NAN)]
#[case(f32::INFINITY)]
#[case(f32::NEG_INFINITY)]
fn rotation_rejects_non_finite(#[case] value: f32) {
    assert!(
        Rotation::new(Degrees(value)).is_none(),
        "Rotation::new(Degrees({value})) must be rejected"
    );
}

#[rstest]
#[case(0.0)]
#[case(45.0)]
#[case(-90.0)]
fn rotation_accepts_valid(#[case] value: f32) {
    assert!(
        Rotation::new(Degrees(value)).is_some(),
        "Rotation::new(Degrees({value})) must be accepted"
    );
}

#[test]
fn rotation_normalizes_negative_zero() {
    assert_neg_zero_equal(
        &Rotation::new(Degrees(-0.0)).expect("Rotation::new should accept -0.0"),
        &Rotation::new(Degrees(0.0)).expect("Rotation::new should accept 0.0"),
        "Rotation",
    );
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "exact float equality is intentional in tests for -0.0 normalization"
)]
fn normalize_float_converts_negative_zero() {
    assert_normalizes!(-0.0, 0.0);
    assert_normalizes!(0.0, 0.0);
    assert_normalizes!(1.0, 1.0);
    assert_normalizes!(-1.0, -1.0);
}
