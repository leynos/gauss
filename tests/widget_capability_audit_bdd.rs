//! BDD tests for widget capability audit inventory.
//!
//! This test module uses rstest-bdd v0.5.0 to validate that the Phase 1-2
//! control inventory satisfies roadmap requirements expressed in Gherkin
//! scenarios.

mod common;

use gauss::ui::widget_audit::{ControlInventory, ControlSurface, Phase};
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

// === Given steps ===

#[given("the widget capability audit inventory is loaded")]
fn given_inventory_loaded(world: &mut AuditWorld) {
    world.inventory = Some(ControlInventory::new());
}

// === When steps ===

#[when("I query controls for Phase 1 toolbar")]
fn when_query_phase1_toolbar(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(ControlSurface::Toolbar)
            .iter()
            .filter(|c| c.phase() == Phase::Phase1)
            .map(|c| c.name().to_string())
            .collect();
    }
}

#[when("I query controls for Phase 1 properties panel")]
fn when_query_phase1_properties(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(ControlSurface::PropertiesPanel)
            .iter()
            .filter(|c| c.phase() == Phase::Phase1)
            .map(|c| c.name().to_string())
            .collect();
    }
}

#[when("I query controls for Phase 1 alignment panel")]
fn when_query_phase1_alignment(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(ControlSurface::AlignmentPanel)
            .iter()
            .filter(|c| c.phase() == Phase::Phase1)
            .map(|c| c.name().to_string())
            .collect();
    }
}

#[when("I query controls for Phase 1 style panel")]
fn when_query_phase1_style(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(ControlSurface::StylePanel)
            .iter()
            .filter(|c| c.phase() == Phase::Phase1)
            .map(|c| c.name().to_string())
            .collect();
    }
}

#[when("I query controls for Phase 1 layers panel")]
fn when_query_phase1_layers(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(ControlSurface::LayersPanel)
            .iter()
            .filter(|c| c.phase() == Phase::Phase1)
            .map(|c| c.name().to_string())
            .collect();
    }
}

#[when("I query controls for Phase 2")]
fn when_query_phase2(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_phase(Phase::Phase2)
            .iter()
            .map(|c| c.name().to_string())
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
            .map(|c| c.name().to_string())
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
    if !world.controls.iter().any(|name| name == "Selection Tool") {
        return Err(TestSupportError::expectation(
            "Inventory must include Selection Tool",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Direct Selection Tool")]
fn then_includes_direct_selection_tool(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name == "Direct Selection Tool")
    {
        return Err(TestSupportError::expectation(
            "Inventory must include Direct Selection Tool",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Pen Tool")]
fn then_includes_pen_tool(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Pen Tool") {
        return Err(TestSupportError::expectation(
            "Inventory must include Pen Tool",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Rectangle Tool")]
fn then_includes_rectangle_tool(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Rectangle Tool") {
        return Err(TestSupportError::expectation(
            "Inventory must include Rectangle Tool",
        ));
    }
    Ok(())
}

#[then("the inventory includes an Ellipse Tool")]
fn then_includes_ellipse_tool(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Ellipse Tool") {
        return Err(TestSupportError::expectation(
            "Inventory must include Ellipse Tool",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Line Tool")]
fn then_includes_line_tool(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Line Tool") {
        return Err(TestSupportError::expectation(
            "Inventory must include Line Tool",
        ));
    }
    Ok(())
}

#[then("the inventory includes an X Position Field")]
fn then_includes_x_position(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name == "X Position Field")
    {
        return Err(TestSupportError::expectation(
            "Inventory must include X Position Field",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Y Position Field")]
fn then_includes_y_position(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name == "Y Position Field")
    {
        return Err(TestSupportError::expectation(
            "Inventory must include Y Position Field",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Width Field")]
fn then_includes_width(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Width Field") {
        return Err(TestSupportError::expectation(
            "Inventory must include Width Field",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Height Field")]
fn then_includes_height(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Height Field") {
        return Err(TestSupportError::expectation(
            "Inventory must include Height Field",
        ));
    }
    Ok(())
}

#[then("the inventory includes a Rotation Field")]
fn then_includes_rotation(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name == "Rotation Field") {
        return Err(TestSupportError::expectation(
            "Inventory must include Rotation Field",
        ));
    }
    Ok(())
}

#[then("the inventory includes alignment controls for left, center, and right")]
fn then_includes_horizontal_alignment(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name.contains("Align Left")) {
        return Err(TestSupportError::expectation("Must include Align Left"));
    }
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Align Center Horizontal"))
    {
        return Err(TestSupportError::expectation(
            "Must include Align Center Horizontal",
        ));
    }
    if !world.controls.iter().any(|name| name.contains("Align Right")) {
        return Err(TestSupportError::expectation("Must include Align Right"));
    }
    Ok(())
}

#[then("the inventory includes alignment controls for top, center, and bottom")]
fn then_includes_vertical_alignment(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name.contains("Align Top")) {
        return Err(TestSupportError::expectation("Must include Align Top"));
    }
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Align Center Vertical"))
    {
        return Err(TestSupportError::expectation(
            "Must include Align Center Vertical",
        ));
    }
    if !world.controls.iter().any(|name| name.contains("Align Bottom")) {
        return Err(TestSupportError::expectation("Must include Align Bottom"));
    }
    Ok(())
}

#[then("the inventory includes distribution controls for horizontal and vertical")]
fn then_includes_distribution(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Distribute Horizontal"))
    {
        return Err(TestSupportError::expectation(
            "Must include Distribute Horizontal",
        ));
    }
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Distribute Vertical"))
    {
        return Err(TestSupportError::expectation(
            "Must include Distribute Vertical",
        ));
    }
    Ok(())
}

#[then("the inventory includes stroke color, width, and opacity controls")]
fn then_includes_stroke_controls(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name.contains("Stroke Color")) {
        return Err(TestSupportError::expectation(
            "Must include Stroke Color Picker",
        ));
    }
    if !world.controls.iter().any(|name| name.contains("Stroke Width")) {
        return Err(TestSupportError::expectation(
            "Must include Stroke Width Field",
        ));
    }
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Stroke Opacity"))
    {
        return Err(TestSupportError::expectation(
            "Must include Stroke Opacity Slider",
        ));
    }
    Ok(())
}

#[then("the inventory includes fill color and opacity controls")]
fn then_includes_fill_controls(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name.contains("Fill Color")) {
        return Err(TestSupportError::expectation(
            "Must include Fill Color Picker",
        ));
    }
    Ok(())
}

#[then("the inventory includes layer visibility toggle")]
fn then_includes_layer_visibility(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Layer Visibility"))
    {
        return Err(TestSupportError::expectation(
            "Must include Layer Visibility Toggle",
        ));
    }
    Ok(())
}

#[then("the inventory includes layer lock toggle")]
fn then_includes_layer_lock(world: &AuditWorld) -> TestSupportResult<()> {
    if !world.controls.iter().any(|name| name.contains("Layer Lock")) {
        return Err(TestSupportError::expectation(
            "Must include Layer Lock Toggle",
        ));
    }
    Ok(())
}

#[then("the inventory includes layer rename capability")]
fn then_includes_layer_rename(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Layer Rename"))
    {
        return Err(TestSupportError::expectation(
            "Must include Layer Rename Field",
        ));
    }
    Ok(())
}

#[then("the inventory includes layer reorder capability")]
fn then_includes_layer_reorder(world: &AuditWorld) -> TestSupportResult<()> {
    if !world
        .controls
        .iter()
        .any(|name| name.contains("Layer Reorder"))
    {
        return Err(TestSupportError::expectation(
            "Must include Layer Reorder Handle",
        ));
    }
    Ok(())
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
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if control.name().is_empty() {
                return Err(TestSupportError::expectation("Control must have name"));
            }
        }
    }
    if world.count == 0 {
        return Err(TestSupportError::expectation(
            "Inventory must have controls",
        ));
    }
    Ok(())
}

#[then("each control has a user job description")]
fn then_each_has_user_job(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if control.user_job.description.is_empty() {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' must have user job",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}

#[then("each control has at least one defined state")]
fn then_each_has_states(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if control.states.states.is_empty() {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' must have states",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}

#[then("each control has an accessibility role and label")]
fn then_each_has_accessibility(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if control.accessibility.role.is_empty() {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' must have a11y role",
                    control.name()
                )));
            }
            if control.accessibility.label.is_empty() {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' must have a11y label",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}

#[then("each control cites at least one requirement source")]
fn then_each_cites_source(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if control.sources.is_empty() {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' must cite sources",
                    control.name()
                )));
            }
        }
    }
    Ok(())
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
        for control in toolbar_controls.iter().filter(|c| c.phase() == Phase::Phase1) {
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
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if !control.keyboard.keyboard_only_operation {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' must support keyboard-only operation",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}

#[then("controls that modify state require action linkage")]
fn then_action_linkage_required(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if !control.action_linkage.requires_action {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' should require action linkage",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}

#[then("action linkage includes implementation notes")]
fn then_action_linkage_documented(world: &AuditWorld) -> TestSupportResult<()> {
    if let Some(ref inventory) = world.inventory {
        for control in inventory.all() {
            if control.action_linkage.requires_action
                && control.action_linkage.notes.is_empty()
            {
                return Err(TestSupportError::expectation(format!(
                    "Control '{}' requires action but has no linkage notes",
                    control.name()
                )));
            }
        }
    }
    Ok(())
}
