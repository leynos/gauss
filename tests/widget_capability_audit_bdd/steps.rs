//! Then step definitions for widget capability audit BDD tests.

use crate::{AuditWorld, assert_each_control, assert_includes, assert_includes_substrings};
use gauss::ui::widget_audit::ControlSurface;
use rstest_bdd_macros::then;
use test_support::TestSupportResult;

/// Inclusion groups for control assertions.
#[derive(Clone, Copy)]
enum InclusionGroup {
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
];

const fn group_checks(group: InclusionGroup) -> &'static [(&'static str, &'static str)] {
    match group {
        InclusionGroup::HorizontalAlignment => H_ALIGN,
        InclusionGroup::VerticalAlignment => V_ALIGN,
        InclusionGroup::StrokeControls => STROKE,
        InclusionGroup::FillControls => FILL,
    }
}

fn assert_includes_group(world: &AuditWorld, group: InclusionGroup) -> TestSupportResult<()> {
    assert_includes_substrings(world, group_checks(group))
}

#[then("the inventory includes a Selection Tool")]
pub(crate) fn then_includes_selection_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Selection Tool")
}

#[then("the inventory includes a Direct Selection Tool")]
pub(crate) fn then_includes_direct_selection(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Direct Selection Tool")
}

#[then("the inventory includes a Pen Tool")]
pub(crate) fn then_includes_pen_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Pen Tool")
}

#[then("the inventory includes a Rectangle Tool")]
pub(crate) fn then_includes_rectangle_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Rectangle Tool")
}

#[then("the inventory includes an Ellipse Tool")]
pub(crate) fn then_includes_ellipse_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Ellipse Tool")
}

#[then("the inventory includes a Line Tool")]
pub(crate) fn then_includes_line_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Line Tool")
}

#[then("the inventory includes an X Position Field")]
pub(crate) fn then_includes_x_position(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "X Position Field")
}

#[then("the inventory includes a Y Position Field")]
pub(crate) fn then_includes_y_position(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Y Position Field")
}

#[then("the inventory includes a Width Field")]
pub(crate) fn then_includes_width_field(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Width Field")
}

#[then("the inventory includes a Height Field")]
pub(crate) fn then_includes_height_field(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Height Field")
}

#[then("the inventory includes a Rotation Field")]
pub(crate) fn then_includes_rotation_field(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Rotation Field")
}

#[then("the inventory includes alignment controls for left, centre, and right")]
pub(crate) fn then_includes_horizontal_align(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_group(world, InclusionGroup::HorizontalAlignment)
}

#[then("the inventory includes alignment controls for top, centre, and bottom")]
pub(crate) fn then_includes_vertical_align(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_group(world, InclusionGroup::VerticalAlignment)
}

#[then("the inventory includes distribution controls for horizontal and vertical")]
pub(crate) fn then_includes_distribution(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            (
                "Distribute Horizontal",
                "Must include Distribute Horizontal",
            ),
            ("Distribute Vertical", "Must include Distribute Vertical"),
        ],
    )
}

#[then("the inventory includes stroke colour, width, and opacity controls")]
pub(crate) fn then_includes_stroke_controls(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_group(world, InclusionGroup::StrokeControls)
}

#[then("the inventory includes fill colour and opacity controls")]
pub(crate) fn then_includes_fill_controls(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_group(world, InclusionGroup::FillControls)
}

#[then("the inventory includes layer visibility toggle")]
pub(crate) fn then_includes_layer_visibility(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Layer Visibility", "Must include Layer Visibility Toggle")],
    )
}

#[then("the inventory includes layer lock toggle")]
pub(crate) fn then_includes_layer_lock(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(world, &[("Layer Lock", "Must include Layer Lock Toggle")])
}

#[then("the inventory includes layer rename capability")]
pub(crate) fn then_includes_layer_rename(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Layer Rename", "Must include Layer Rename Field")],
    )
}

#[then("the inventory includes layer reorder capability")]
pub(crate) fn then_includes_layer_reorder(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Layer Reorder", "Must include Layer Reorder Handle")],
    )
}

#[then("the inventory includes character panel controls")]
pub(crate) fn then_includes_character_panel(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            ("Font Family", "Must include Font Family Selector"),
            ("Font Size", "Must include Font Size Field"),
            ("Bold", "Must include Bold Toggle"),
            ("Italic", "Must include Italic Toggle"),
            ("Text Alignment", "Must include Text Alignment Buttons"),
            ("Text Colour", "Must include Text Colour Picker"),
        ],
    )
}

#[then("the inventory includes paragraph panel controls")]
pub(crate) fn then_includes_paragraph_panel(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            ("Paragraph Spacing", "Must include Paragraph Spacing Field"),
            ("Line Spacing", "Must include Line Spacing Field"),
            ("Indentation", "Must include Indentation Controls"),
        ],
    )
}

#[then("the inventory includes canvas text editing controls")]
pub(crate) fn then_includes_canvas_text(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Text Cursor") || name.contains("Inline Text"))
    {
        return Err(test_support::TestSupportError::expectation(
            "Must include canvas text editing controls",
        ));
    }
    Ok(())
}

#[then("the inventory includes undo and redo controls")]
pub(crate) fn then_includes_undo_redo(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            ("Undo", "Must include Undo control"),
            ("Redo", "Must include Redo control"),
        ],
    )
}

#[then("the inventory includes history clear capability")]
pub(crate) fn then_includes_history_clear(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(world, &[("Clear History", "Must include History Clear")])
}

#[then("the inventory includes text insertion capability")]
pub(crate) fn then_includes_text_insertion(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Text Insertion", "Must include text insertion capability")],
    )
}

#[then("the inventory includes text selection capability")]
pub(crate) fn then_includes_text_selection(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Text Selection", "Must include text selection capability")],
    )
}

#[then("each control has a non-empty name")]
pub(crate) fn then_each_has_name(world: &AuditWorld) -> TestSupportResult<()> {
    if world.count == 0 {
        return Err(test_support::TestSupportError::expectation(
            "Inventory must have controls",
        ));
    }
    assert_each_control(
        world,
        |c| !c.name().is_empty(),
        |_| "Control must have name".to_owned(),
    )
}

#[then("each control has a user job description")]
pub(crate) fn then_each_has_user_job(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.user_job.description.is_empty(),
        |c| format!("Control '{}' must have user job", c.name()),
    )
}

#[then("each control has at least one defined state")]
pub(crate) fn then_each_has_states(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.states.states.is_empty(),
        |c| format!("Control '{}' must have states", c.name()),
    )
}

#[then("each control has an accessibility role and label")]
pub(crate) fn then_each_has_accessibility(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.accessibility.role.is_empty(),
        |c| format!("Control '{}' must have a11y role", c.name()),
    )?;
    assert_each_control(
        world,
        |c| !c.accessibility.label.is_empty(),
        |c| format!("Control '{}' must have a11y label", c.name()),
    )
}

#[then("each control cites at least one requirement source")]
pub(crate) fn then_each_cites_source(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.sources.is_empty(),
        |c| format!("Control '{}' must cite sources", c.name()),
    )
}

#[then("at least one control has evidence")]
pub(crate) fn then_at_least_one_evidence(world: &AuditWorld) -> TestSupportResult<()> {
    let inventory = world.inventory.as_ref().ok_or_else(|| {
        test_support::TestSupportError::expectation("Inventory not loaded".to_owned())
    })?;
    if inventory.with_evidence().is_empty() {
        return Err(test_support::TestSupportError::expectation(
            "At least one control must have shell evidence",
        ));
    }
    Ok(())
}

#[then("each control with evidence references a source file path")]
pub(crate) fn then_evidence_has_path(world: &AuditWorld) -> TestSupportResult<()> {
    let inventory = world.inventory.as_ref().ok_or_else(|| {
        test_support::TestSupportError::expectation("Inventory not loaded".to_owned())
    })?;
    for control in inventory.with_evidence() {
        if control.current_evidence.file_path.is_none() {
            return Err(test_support::TestSupportError::expectation(format!(
                "Control '{}' claims evidence but has no file path",
                control.name()
            )));
        }
    }
    Ok(())
}

#[then("each tool has a keyboard shortcut defined")]
pub(crate) fn then_each_tool_has_shortcut(world: &AuditWorld) -> TestSupportResult<()> {
    let inventory = world
        .inventory
        .as_ref()
        .ok_or_else(|| test_support::TestSupportError::expectation("Inventory must be loaded"))?;

    let tools: Vec<_> = inventory
        .by_surface(ControlSurface::Toolbar)
        .into_iter()
        .collect();

    if tools.is_empty() {
        return Err(test_support::TestSupportError::expectation(
            "Must have toolbar controls".to_owned(),
        ));
    }

    if let Some(missing) = tools.iter().find(|c| c.keyboard.shortcut.is_none()) {
        return Err(test_support::TestSupportError::expectation(format!(
            "Toolbar tool '{}' must have keyboard shortcut",
            missing.name()
        )));
    }

    Ok(())
}

#[then("each control supports keyboard-only operation")]
pub(crate) fn then_each_supports_keyboard(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| c.keyboard.keyboard_only_operation,
        |c| {
            format!(
                "Control '{}' must support keyboard-only operation",
                c.name()
            )
        },
    )
}

#[then("all controls require action linkage")]
pub(crate) fn then_action_linkage_required(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| c.action_linkage.requires_action,
        |c| format!("Control '{}' should require action linkage", c.name()),
    )
}

#[then("action linkage includes implementation notes")]
pub(crate) fn then_action_linkage_documented(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !(c.action_linkage.requires_action && c.action_linkage.notes.is_empty()),
        |c| {
            format!(
                "Control '{}' requires action but has no linkage notes",
                c.name()
            )
        },
    )
}
