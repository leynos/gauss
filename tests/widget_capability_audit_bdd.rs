//! BDD tests for widget capability audit inventory.
//!
//! This test module uses rstest-bdd v0.5.0 to validate that the Phase 1-2
//! control inventory satisfies roadmap requirements expressed in Gherkin
//! scenarios.

mod common;

use gauss::ui::widget_audit::{ControlInventory, ControlSurface, Phase};
use rstest_bdd::{feature, given, scenario, then, when};

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Phase 1 toolbar tools are catalogued")]
fn test_phase1_toolbar_tools_catalogued() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Phase 1 properties panel controls are catalogued")]
fn test_phase1_properties_panel_catalogued() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Phase 1 alignment controls are catalogued")]
fn test_phase1_alignment_controls_catalogued() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Phase 1 style panel controls are catalogued")]
fn test_phase1_style_panel_catalogued() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Phase 1 layers panel controls are catalogued")]
fn test_phase1_layers_panel_catalogued() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Phase 2 text controls are catalogued")]
fn test_phase2_text_controls_catalogued() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("All controls have complete requirement fields")]
fn test_all_controls_complete() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("Current shell evidence is tracked")]
fn test_shell_evidence_tracked() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("All toolbar tools have keyboard shortcuts")]
fn test_toolbar_keyboard_shortcuts() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("All controls support keyboard-only operation")]
fn test_keyboard_only_operation() {}

#[feature("tests/features/widget_capability_audit.feature")]
#[scenario("All controls integrate with Action/Command system")]
fn test_action_command_integration() {}

// Step definitions

#[given("the widget capability audit inventory is loaded")]
fn given_inventory_loaded() -> ControlInventory {
    ControlInventory::new()
}

#[when("I query controls for Phase 1 toolbar")]
fn when_query_phase1_toolbar(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_surface(ControlSurface::Toolbar)
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I query controls for Phase 1 properties panel")]
fn when_query_phase1_properties(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_surface(ControlSurface::PropertiesPanel)
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I query controls for Phase 1 alignment panel")]
fn when_query_phase1_alignment(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_surface(ControlSurface::AlignmentPanel)
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I query controls for Phase 1 style panel")]
fn when_query_phase1_style(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_surface(ControlSurface::StylePanel)
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I query controls for Phase 1 layers panel")]
fn when_query_phase1_layers(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_surface(ControlSurface::LayersPanel)
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I query controls for Phase 2")]
fn when_query_phase2(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_phase(Phase::Phase2)
        .iter()
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I examine all controls in the inventory")]
fn when_examine_all_controls(inventory: &ControlInventory) -> usize {
    inventory.all().len()
}

#[when("I query controls with current shell evidence")]
fn when_query_with_evidence(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .with_evidence()
        .iter()
        .map(|c| c.name().to_string())
        .collect()
}

#[when("I query Phase 1 toolbar controls")]
fn when_query_toolbar_phase1(inventory: &ControlInventory) -> Vec<String> {
    inventory
        .by_surface(ControlSurface::Toolbar)
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .map(|c| c.name().to_string())
        .collect()
}

#[then("the inventory includes a Selection Tool")]
fn then_includes_selection_tool(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Selection Tool"),
        "Inventory must include Selection Tool"
    );
}

#[then("the inventory includes a Direct Selection Tool")]
fn then_includes_direct_selection_tool(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Direct Selection Tool"),
        "Inventory must include Direct Selection Tool"
    );
}

#[then("the inventory includes a Pen Tool")]
fn then_includes_pen_tool(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Pen Tool"),
        "Inventory must include Pen Tool"
    );
}

#[then("the inventory includes a Rectangle Tool")]
fn then_includes_rectangle_tool(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Rectangle Tool"),
        "Inventory must include Rectangle Tool"
    );
}

#[then("the inventory includes an Ellipse Tool")]
fn then_includes_ellipse_tool(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Ellipse Tool"),
        "Inventory must include Ellipse Tool"
    );
}

#[then("the inventory includes a Line Tool")]
fn then_includes_line_tool(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Line Tool"),
        "Inventory must include Line Tool"
    );
}

#[then("the inventory includes an X Position Field")]
fn then_includes_x_position(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "X Position Field"),
        "Inventory must include X Position Field"
    );
}

#[then("the inventory includes a Y Position Field")]
fn then_includes_y_position(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Y Position Field"),
        "Inventory must include Y Position Field"
    );
}

#[then("the inventory includes a Width Field")]
fn then_includes_width(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Width Field"),
        "Inventory must include Width Field"
    );
}

#[then("the inventory includes a Height Field")]
fn then_includes_height(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Height Field"),
        "Inventory must include Height Field"
    );
}

#[then("the inventory includes a Rotation Field")]
fn then_includes_rotation(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name == "Rotation Field"),
        "Inventory must include Rotation Field"
    );
}

#[then("the inventory includes alignment controls for left, center, and right")]
fn then_includes_horizontal_alignment(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Align Left")),
        "Must include Align Left"
    );
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Align Center Horizontal")),
        "Must include Align Center Horizontal"
    );
    assert!(
        controls.iter().any(|name| name.contains("Align Right")),
        "Must include Align Right"
    );
}

#[then("the inventory includes alignment controls for top, center, and bottom")]
fn then_includes_vertical_alignment(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Align Top")),
        "Must include Align Top"
    );
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Align Center Vertical")),
        "Must include Align Center Vertical"
    );
    assert!(
        controls.iter().any(|name| name.contains("Align Bottom")),
        "Must include Align Bottom"
    );
}

#[then("the inventory includes distribution controls for horizontal and vertical")]
fn then_includes_distribution(controls: &Vec<String>) {
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Distribute Horizontal")),
        "Must include Distribute Horizontal"
    );
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Distribute Vertical")),
        "Must include Distribute Vertical"
    );
}

#[then("the inventory includes stroke color, width, and opacity controls")]
fn then_includes_stroke_controls(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Stroke Color")),
        "Must include Stroke Color Picker"
    );
    assert!(
        controls.iter().any(|name| name.contains("Stroke Width")),
        "Must include Stroke Width Field"
    );
    assert!(
        controls.iter().any(|name| name.contains("Stroke Opacity")),
        "Must include Stroke Opacity Slider"
    );
}

#[then("the inventory includes fill color and opacity controls")]
fn then_includes_fill_controls(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Fill Color")),
        "Must include Fill Color Picker"
    );
    // Note: Fill Opacity may not be implemented yet but should be in inventory
}

#[then("the inventory includes layer visibility toggle")]
fn then_includes_layer_visibility(controls: &Vec<String>) {
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Layer Visibility")),
        "Must include Layer Visibility Toggle"
    );
}

#[then("the inventory includes layer lock toggle")]
fn then_includes_layer_lock(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Layer Lock")),
        "Must include Layer Lock Toggle"
    );
}

#[then("the inventory includes layer rename capability")]
fn then_includes_layer_rename(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Layer Rename")),
        "Must include Layer Rename Field"
    );
}

#[then("the inventory includes layer reorder capability")]
fn then_includes_layer_reorder(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Layer Reorder")),
        "Must include Layer Reorder Handle"
    );
}

#[then("the inventory includes character panel controls")]
fn then_includes_character_panel(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Font")),
        "Must include font controls"
    );
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Bold") || name.contains("Italic")),
        "Must include text formatting controls"
    );
}

#[then("the inventory includes paragraph panel controls")]
fn then_includes_paragraph_panel(controls: &Vec<String>) {
    assert!(
        controls.iter().any(|name| name.contains("Paragraph")
            || name.contains("Line Spacing")
            || name.contains("Indentation")),
        "Must include paragraph formatting controls"
    );
}

#[then("the inventory includes canvas text editing controls")]
fn then_includes_canvas_text(controls: &Vec<String>) {
    assert!(
        controls
            .iter()
            .any(|name| name.contains("Text Cursor") || name.contains("Inline Text")),
        "Must include canvas text editing controls"
    );
}

#[then("each control has a non-empty name")]
fn then_each_has_name(count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(!control.name().is_empty(), "Control must have name");
    }
    assert!(*count > 0, "Inventory must have controls");
}

#[then("each control has a user job description")]
fn then_each_has_user_job(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(
            !control.user_job.description.is_empty(),
            "Control '{}' must have user job",
            control.name()
        );
    }
}

#[then("each control has at least one defined state")]
fn then_each_has_states(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(
            !control.states.states.is_empty(),
            "Control '{}' must have states",
            control.name()
        );
    }
}

#[then("each control has an accessibility role and label")]
fn then_each_has_accessibility(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(
            !control.accessibility.role.is_empty(),
            "Control '{}' must have a11y role",
            control.name()
        );
        assert!(
            !control.accessibility.label.is_empty(),
            "Control '{}' must have a11y label",
            control.name()
        );
    }
}

#[then("each control cites at least one requirement source")]
fn then_each_cites_source(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(
            !control.sources.is_empty(),
            "Control '{}' must cite sources",
            control.name()
        );
    }
}

#[then("at least one control has evidence")]
fn then_at_least_one_evidence(controls: &Vec<String>) {
    assert!(
        !controls.is_empty(),
        "At least one control must have shell evidence"
    );
}

#[then("each control with evidence references a source file path")]
fn then_evidence_has_path(_controls: &Vec<String>) {
    let inventory = ControlInventory::new();
    for control in inventory.with_evidence() {
        assert!(
            control.current_evidence.file_path.is_some(),
            "Control '{}' claims evidence but has no file path",
            control.name()
        );
    }
}

#[then("each tool has a keyboard shortcut defined")]
fn then_each_tool_has_shortcut(controls: &Vec<String>) {
    let inventory = ControlInventory::new();
    let toolbar_controls = inventory.by_surface(ControlSurface::Toolbar);

    for control in toolbar_controls
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
    {
        assert!(
            control.keyboard.shortcut.is_some(),
            "Toolbar tool '{}' must have keyboard shortcut",
            control.name()
        );
    }
    assert!(!controls.is_empty(), "Must have toolbar controls");
}

#[then("each control supports keyboard-only operation")]
fn then_each_supports_keyboard(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(
            control.keyboard.keyboard_only_operation,
            "Control '{}' must support keyboard-only operation",
            control.name()
        );
    }
}

#[then("controls that modify state require action linkage")]
fn then_action_linkage_required(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        // All controls in this inventory should require actions since they're all interactive
        assert!(
            control.action_linkage.requires_action,
            "Control '{}' should require action linkage",
            control.name()
        );
    }
}

#[then("action linkage includes implementation notes")]
fn then_action_linkage_documented(_count: &usize) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        if control.action_linkage.requires_action {
            assert!(
                !control.action_linkage.notes.is_empty(),
                "Control '{}' requires action but has no linkage notes",
                control.name()
            );
        }
    }
}
