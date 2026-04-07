//! Tests for `action_payloads` module.

use std::hash::{Hash, Hasher};

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

/// Macro for asserting hash and value equality of two constructed instances.
macro_rules! assert_neg_zero_equal {
    ($neg_expr:expr, $pos_expr:expr $(, $ctx:expr)?) => {{
        let a = $neg_expr;
        let b = $pos_expr;
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(
            h1.finish(),
            h2.finish(),
            "hash must be equal for -0.0 vs +0.0{}",
            concat!($(", ", $ctx)?)
        );
        assert_eq!(
            a,
            b,
            "value must be equal for -0.0 vs +0.0{}",
            concat!($(", ", $ctx)?)
        );
    }};
}

/// Helper for asserting that `StrokeWidth::new` rejects invalid values.
fn assert_stroke_width_rejected(points: Points) {
    assert!(
        StrokeWidth::new(points).is_none(),
        "StrokeWidth::new(Points({})) must be rejected",
        points.0
    );
}

/// Helper for asserting that `StrokeWidth::new` accepts valid values.
fn assert_stroke_width_accepted(points: Points) {
    assert!(
        StrokeWidth::new(points).is_some(),
        "StrokeWidth::new(Points({})) must be accepted",
        points.0
    );
}

/// Helper for asserting that `Position::new` rejects invalid points.
fn assert_position_rejected(p: Point) {
    assert!(
        Position::new(p).is_none(),
        "Position::new(Point {{ x: {}, y: {} }}) must be rejected",
        p.x,
        p.y
    );
}

/// Helper for asserting that `Position::new` accepts valid points.
fn assert_position_accepted(p: Point) {
    assert!(
        Position::new(p).is_some(),
        "Position::new(Point {{ x: {}, y: {} }}) must be accepted",
        p.x,
        p.y
    );
}

/// Helper for asserting that `Size::new` rejects invalid dimensions.
fn assert_size_rejected(d: Dimensions) {
    assert!(
        Size::new(d).is_none(),
        "Size::new(Dimensions {{ width: {}, height: {} }}) must be rejected",
        d.width,
        d.height
    );
}

/// Helper for asserting that `Size::new` accepts valid dimensions.
fn assert_size_accepted(d: Dimensions) {
    assert!(
        Size::new(d).is_some(),
        "Size::new(Dimensions {{ width: {}, height: {} }}) must be accepted",
        d.width,
        d.height
    );
}

#[test]
fn stroke_width_rejects_negative_and_non_finite() {
    // Negative should be rejected
    assert_stroke_width_rejected(Points(-1.0));
    assert_stroke_width_rejected(Points(-0.1));

    // Non-finite should be rejected
    assert_stroke_width_rejected(Points(f32::NAN));
    assert_stroke_width_rejected(Points(f32::INFINITY));
    assert_stroke_width_rejected(Points(f32::NEG_INFINITY));

    // Valid values should be accepted
    assert_stroke_width_accepted(Points(0.0));
    assert_stroke_width_accepted(Points(1.0));
    assert_stroke_width_accepted(Points(2.5));
}

#[test]
fn stroke_width_normalizes_negative_zero() {
    assert_neg_zero_equal!(
        StrokeWidth::new(Points(-0.0)).expect("StrokeWidth::new should accept -0.0"),
        StrokeWidth::new(Points(0.0)).expect("StrokeWidth::new should accept 0.0"),
        "StrokeWidth"
    );
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
    assert_position_rejected(Point {
        x: f32::NAN,
        y: 0.0,
    });
    assert_position_rejected(Point {
        x: 0.0,
        y: f32::NAN,
    });
    assert_position_rejected(Point {
        x: f32::INFINITY,
        y: 0.0,
    });
    assert_position_rejected(Point {
        x: 0.0,
        y: f32::NEG_INFINITY,
    });

    // Valid values should be accepted
    assert_position_accepted(Point { x: 0.0, y: 0.0 });
    assert_position_accepted(Point { x: 1.0, y: 2.0 });
}

#[test]
fn position_normalizes_negative_zero() {
    assert_neg_zero_equal!(
        Position::new(Point { x: -0.0, y: 5.0 }).expect("Position::new should accept -0.0"),
        Position::new(Point { x: 0.0, y: 5.0 }).expect("Position::new should accept 0.0"),
        "Position(x)"
    );
}

#[test]
fn size_rejects_negative_and_non_finite() {
    // Negative dimensions should be rejected
    assert_size_rejected(Dimensions {
        width: -1.0,
        height: 1.0,
    });
    assert_size_rejected(Dimensions {
        width: 1.0,
        height: -1.0,
    });
    assert_size_rejected(Dimensions {
        width: -0.1,
        height: -0.1,
    });

    // Non-finite should be rejected
    assert_size_rejected(Dimensions {
        width: f32::NAN,
        height: 1.0,
    });
    assert_size_rejected(Dimensions {
        width: 1.0,
        height: f32::INFINITY,
    });
    assert_size_rejected(Dimensions {
        width: f32::NEG_INFINITY,
        height: 1.0,
    });

    // Valid values should be accepted
    assert_size_accepted(Dimensions {
        width: 0.0,
        height: 0.0,
    });
    assert_size_accepted(Dimensions {
        width: 100.0,
        height: 200.0,
    });
}

#[test]
fn size_normalizes_negative_zero() {
    assert_neg_zero_equal!(
        Size::new(Dimensions {
            width: -0.0,
            height: 10.0
        })
        .expect("Size::new should accept -0.0"),
        Size::new(Dimensions {
            width: 0.0,
            height: 10.0
        })
        .expect("Size::new should accept 0.0"),
        "Size(width)"
    );
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
    assert_neg_zero_equal!(
        Rotation::new(Degrees(-0.0)).expect("Rotation::new should accept -0.0"),
        Rotation::new(Degrees(0.0)).expect("Rotation::new should accept 0.0"),
        "Rotation"
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
