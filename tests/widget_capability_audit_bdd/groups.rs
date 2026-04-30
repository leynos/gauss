//! Inclusion groups for control assertions.

use crate::{AuditWorld, assert_includes_substrings};
use test_support::TestSupportResult;

/// Inclusion groups for control assertions.
#[derive(Clone, Copy)]
pub(crate) enum InclusionGroup {
    HorizontalAlignment,
    VerticalAlignment,
    StrokeControls,
    FillControls,
}

const H_ALIGN: &[(&str, &str)] = &[
    ("Align Left", "Must include Align Left"),
    (
        "Align Centre Horizontal",
        "Must include Align Centre Horizontal",
    ),
    ("Align Right", "Must include Align Right"),
];

const V_ALIGN: &[(&str, &str)] = &[
    ("Align Top", "Must include Align Top"),
    (
        "Align Centre Vertical",
        "Must include Align Centre Vertical",
    ),
    ("Align Bottom", "Must include Align Bottom"),
];

const STROKE: &[(&str, &str)] = &[
    ("Stroke Colour", "Must include Stroke Colour Picker"),
    ("Stroke Width", "Must include Stroke Width Field"),
    ("Stroke Opacity", "Must include Stroke Opacity Slider"),
];

const FILL: &[(&str, &str)] = &[
    ("Fill Colour", "Must include Fill Colour Picker"),
    ("Fill Opacity", "Must include Fill Opacity Slider"),
    ("No Fill Toggle", "Must include No Fill Toggle"),
];

const fn group_checks(group: InclusionGroup) -> &'static [(&'static str, &'static str)] {
    match group {
        InclusionGroup::HorizontalAlignment => H_ALIGN,
        InclusionGroup::VerticalAlignment => V_ALIGN,
        InclusionGroup::StrokeControls => STROKE,
        InclusionGroup::FillControls => FILL,
    }
}

pub(crate) fn assert_includes_group(
    world: &AuditWorld,
    group: InclusionGroup,
) -> TestSupportResult<()> {
    assert_includes_substrings(world, group_checks(group))
}
