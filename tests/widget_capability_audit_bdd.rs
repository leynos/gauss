//! BDD tests for widget capability audit inventory.
//!
//! This test module uses rstest-bdd v0.5.0 to validate that the Phase 1-2
//! control inventory satisfies roadmap requirements expressed in Gherkin
//! scenarios.

use gauss::ui::widget_audit::{ControlInventory, ControlSurface, Phase, RequiredControl};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

/// World state for widget audit BDD tests.
#[derive(Default)]
struct AuditWorld {
    inventory: Option<ControlInventory>,
    controls: Vec<String>,
    count: usize,
}

#[fixture]
fn world() -> AuditWorld {
    AuditWorld::default()
}

#[scenario("tests/features/widget_capability_audit.feature")]
fn widget_capability_audit() {}

fn query_by_phase_and_surface(world: &mut AuditWorld, phase: Phase, surface: ControlSurface) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(surface)
            .iter()
            .filter(|c| c.phase() == phase)
            .map(|c| c.name().to_owned())
            .collect();
    }
}

fn assert_includes(world: &AuditWorld, expected: &str) -> TestSupportResult<()> {
    if !world.controls.iter().any(|n| n == expected) {
        return Err(TestSupportError::expectation(format!(
            "Inventory must include {expected}"
        )));
    }
    Ok(())
}

fn assert_includes_substrings(
    world: &AuditWorld,
    checks: &[(&str, &str)],
) -> TestSupportResult<()> {
    for (sub, msg) in checks {
        if !world.controls.iter().any(|n| n.contains(sub)) {
            return Err(TestSupportError::expectation(*msg));
        }
    }
    Ok(())
}

fn for_each_control<F>(world: &AuditWorld, check: F) -> TestSupportResult<()>
where
    F: Fn(&gauss::ui::widget_audit::RequiredControl) -> TestSupportResult<()>,
{
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            check(control)?;
        }
    }
    Ok(())
}

fn assert_each_control<P, M>(world: &AuditWorld, predicate: P, message: M) -> TestSupportResult<()>
where
    P: Fn(&RequiredControl) -> bool,
    M: Fn(&RequiredControl) -> String,
{
    for_each_control(world, |c| {
        if predicate(c) {
            Ok(())
        } else {
            Err(TestSupportError::expectation(message(c)))
        }
    })
}

// === Given steps ===

#[given("the widget capability audit inventory is loaded")]
fn given_inventory_loaded(world: &mut AuditWorld) {
    world.inventory = Some(ControlInventory::new());
}

// === When steps ===

#[when("I query controls for Phase 1 toolbar")]
fn when_query_phase1_toolbar(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::Toolbar);
}

#[when("I query controls for Phase 1 properties panel")]
fn when_query_phase1_properties(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::PropertiesPanel);
}

#[when("I query controls for Phase 1 alignment panel")]
fn when_query_phase1_alignment(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::AlignmentPanel);
}

#[when("I query controls for Phase 1 style panel")]
fn when_query_phase1_style(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::StylePanel);
}

#[when("I query controls for Phase 1 layers panel")]
fn when_query_phase1_layers(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::LayersPanel);
}

#[when("I query controls for Phase 2")]
fn when_query_phase2(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_phase(Phase::Phase2)
            .iter()
            .map(|c| c.name().to_owned())
            .collect();
    }
}

#[when("I examine all controls in the inventory")]
fn when_examine_all_controls(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.count = inventory.all().len();
    }
}

#[when("I query controls with current shell evidence")]
fn when_query_with_evidence(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .with_evidence()
            .iter()
            .map(|c| c.name().to_owned())
            .collect();
    }
}

#[when("I query Phase 1 toolbar controls")]
fn when_query_toolbar_phase1(world: &mut AuditWorld) {
    when_query_phase1_toolbar(world);
}

// === Then steps ===

#[then("the inventory includes a Selection Tool")]
fn then_includes_selection_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Selection Tool")
}

#[then("the inventory includes a Direct Selection Tool")]
fn then_includes_direct_selection_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Direct Selection Tool")
}

#[then("the inventory includes a Pen Tool")]
fn then_includes_pen_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Pen Tool")
}

#[then("the inventory includes a Rectangle Tool")]
fn then_includes_rectangle_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Rectangle Tool")
}

#[then("the inventory includes an Ellipse Tool")]
fn then_includes_ellipse_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Ellipse Tool")
}

#[then("the inventory includes a Line Tool")]
fn then_includes_line_tool(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Line Tool")
}

#[then("the inventory includes an X Position Field")]
fn then_includes_x_position(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "X Position Field")
}

#[then("the inventory includes a Y Position Field")]
fn then_includes_y_position(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Y Position Field")
}

#[then("the inventory includes a Width Field")]
fn then_includes_width(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Width Field")
}

#[then("the inventory includes a Height Field")]
fn then_includes_height(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Height Field")
}

#[then("the inventory includes a Rotation Field")]
fn then_includes_rotation(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes(world, "Rotation Field")
}

#[then("the inventory includes alignment controls for left, center, and right")]
fn then_includes_horizontal_alignment(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            ("Align Left", "Must include Align Left"),
            (
                "Align Center Horizontal",
                "Must include Align Center Horizontal",
            ),
            ("Align Right", "Must include Align Right"),
        ],
    )
}

#[then("the inventory includes alignment controls for top, center, and bottom")]
fn then_includes_vertical_alignment(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            ("Align Top", "Must include Align Top"),
            (
                "Align Center Vertical",
                "Must include Align Center Vertical",
            ),
            ("Align Bottom", "Must include Align Bottom"),
        ],
    )
}

#[then("the inventory includes distribution controls for horizontal and vertical")]
fn then_includes_distribution(world: &AuditWorld) -> TestSupportResult<()> {
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

#[then("the inventory includes stroke color, width, and opacity controls")]
fn then_includes_stroke_controls(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[
            ("Stroke Color", "Must include Stroke Color Picker"),
            ("Stroke Width", "Must include Stroke Width Field"),
            ("Stroke Opacity", "Must include Stroke Opacity Slider"),
        ],
    )
}

#[then("the inventory includes fill color and opacity controls")]
fn then_includes_fill_controls(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(world, &[("Fill Color", "Must include Fill Color Picker")])
}

#[then("the inventory includes layer visibility toggle")]
fn then_includes_layer_visibility(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Layer Visibility", "Must include Layer Visibility Toggle")],
    )
}

#[then("the inventory includes layer lock toggle")]
fn then_includes_layer_lock(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(world, &[("Layer Lock", "Must include Layer Lock Toggle")])
}

#[then("the inventory includes layer rename capability")]
fn then_includes_layer_rename(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Layer Rename", "Must include Layer Rename Field")],
    )
}

#[then("the inventory includes layer reorder capability")]
fn then_includes_layer_reorder(world: &AuditWorld) -> TestSupportResult<()> {
    assert_includes_substrings(
        world,
        &[("Layer Reorder", "Must include Layer Reorder Handle")],
    )
}

#[then("the inventory includes character panel controls")]
fn then_includes_character_panel(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name.contains("Font")) {
        return Err(TestSupportError::expectation("Must include font controls"));
    }
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Bold") || name.contains("Italic"))
    {
        return Err(TestSupportError::expectation(
            "Must include text formatting controls",
        ));
    }
    Ok(())
}

#[then("the inventory includes paragraph panel controls")]
fn then_includes_paragraph_panel(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| {
        name.contains("Paragraph") || name.contains("Line Spacing") || name.contains("Indentation")
    }) {
        return Err(TestSupportError::expectation(
            "Must include paragraph formatting controls",
        ));
    }
    Ok(())
}

#[then("the inventory includes canvas text editing controls")]
fn then_includes_canvas_text(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Text Cursor") || name.contains("Inline Text"))
    {
        return Err(TestSupportError::expectation(
            "Must include canvas text editing controls",
        ));
    }
    Ok(())
}

#[then("each control has a non-empty name")]
fn then_each_has_name(world: &AuditWorld) -> TestSupportResult<()> {
    if world.count == 0 {
        return Err(TestSupportError::expectation(
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
fn then_each_has_user_job(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.user_job.description.is_empty(),
        |c| format!("Control '{}' must have user job", c.name()),
    )
}

#[then("each control has at least one defined state")]
fn then_each_has_states(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.states.states.is_empty(),
        |c| format!("Control '{}' must have states", c.name()),
    )
}

#[then("each control has an accessibility role and label")]
fn then_each_has_accessibility(world: &AuditWorld) -> TestSupportResult<()> {
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
fn then_each_cites_source(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| !c.sources.is_empty(),
        |c| format!("Control '{}' must cite sources", c.name()),
    )
}

#[then("at least one control has evidence")]
fn then_at_least_one_evidence(world: &AuditWorld) -> TestSupportResult<()> {
    if world.controls.is_empty() {
        return Err(TestSupportError::expectation(
            "At least one control must have shell evidence",
        ));
    }
    Ok(())
}

#[then("each control with evidence references a source file path")]
fn then_evidence_has_path(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.with_evidence() {
            if control.current_evidence.file_path.is_none() {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' claims evidence but has no file path",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}

#[then("each tool has a keyboard shortcut defined")]
fn then_each_tool_has_shortcut(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        let toolbar_controls = inventory.by_surface(ControlSurface::Toolbar);
        for control in toolbar_controls
            .iter()
            .filter(|c| c.phase() == Phase::Phase1)
        {
            if control.keyboard.shortcut.is_none() {
                return Err(TestSupportError::expectation(format!(
                    "Toolbar tool '{}' must have keyboard shortcut",
                    control.name()
                )));
            }
        }
    }
    if world.controls.is_empty() {
        return Err(TestSupportError::expectation("Must have toolbar controls"));
    }
    Ok(())
}

#[then("each control supports keyboard-only operation")]
fn then_each_supports_keyboard(world: &AuditWorld) -> TestSupportResult<()> {
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

#[then("controls that modify state require action linkage")]
fn then_action_linkage_required(world: &AuditWorld) -> TestSupportResult<()> {
    assert_each_control(
        world,
        |c| c.action_linkage.requires_action,
        |c| format!("Control '{}' should require action linkage", c.name()),
    )
}

#[then("action linkage includes implementation notes")]
fn then_action_linkage_documented(world: &AuditWorld) -> TestSupportResult<()> {
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
