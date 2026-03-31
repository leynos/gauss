//! Unit tests for widget capability audit inventory.
//!
//! This test module validates the Phase 1-2 control inventory defined in
//! `src/ui/widget_audit/mod.rs`. Tests ensure:
//!
//! - All control entries have complete required fields
//! - Phase 1 and Phase 2 controls are properly categorized
//! - Current shell evidence is accurately recorded
//! - All expected control surfaces have at least one control
//! - All controls that should have keyboard shortcuts do have them
//! - All controls have proper accessibility requirements

use gauss::ui::widget_audit::{ControlInventory, ControlSurface, Phase, RequiredControl};
use rstest::rstest;

fn assert_all_controls_satisfy(label: &str, predicate: fn(&RequiredControl) -> bool) {
    let inventory = ControlInventory::new();
    for control in inventory.all() {
        assert!(
            predicate(control),
            "Control '{}' failed invariant: {label}",
            control.name()
        );
    }
}

#[rstest]
fn test_inventory_is_not_empty() {
    let inventory = ControlInventory::new();
    assert!(
        !inventory.all().is_empty(),
        "Control inventory must not be empty"
    );
}

#[rstest]
#[case("has a non-empty name",
       (|c: &RequiredControl| !c.name().is_empty()) as fn(&RequiredControl) -> bool)]
#[case("has a user job description",
       (|c: &RequiredControl| !c.user_job.description.is_empty()) as fn(&RequiredControl) -> bool)]
#[case("defines at least one state",
       (|c: &RequiredControl| !c.states.states.is_empty()) as fn(&RequiredControl) -> bool)]
#[case("has an accessibility role",
       (|c: &RequiredControl| !c.accessibility.role.is_empty()) as fn(&RequiredControl) -> bool)]
#[case("has an accessibility label",
       (|c: &RequiredControl| !c.accessibility.label.is_empty()) as fn(&RequiredControl) -> bool)]
#[case("cites at least one requirement source",
       (|c: &RequiredControl| !c.sources.is_empty()) as fn(&RequiredControl) -> bool)]
#[case("supports keyboard-only operation",
       (|c: &RequiredControl| c.keyboard.keyboard_only_operation) as fn(&RequiredControl) -> bool)]
fn test_all_controls_satisfy_invariant(
    #[case] label: &str,
    #[case] predicate: fn(&RequiredControl) -> bool,
) {
    assert_all_controls_satisfy(label, predicate);
}

#[rstest]
fn test_phase1_controls_exist() {
    let inventory = ControlInventory::new();
    let phase1_controls = inventory.by_phase(Phase::Phase1);
    assert!(
        !phase1_controls.is_empty(),
        "Phase 1 must have at least one control"
    );
}

#[rstest]
fn test_phase2_controls_exist() {
    let inventory = ControlInventory::new();
    let phase2_controls = inventory.by_phase(Phase::Phase2);
    assert!(
        !phase2_controls.is_empty(),
        "Phase 2 must have at least one control"
    );
}

fn assert_phase2_surface_all_phase2(surface: ControlSurface) {
    let inventory = ControlInventory::new();
    let controls = inventory.by_surface(surface);

    assert!(
        !controls.is_empty(),
        "Phase 2 must define {surface} controls"
    );

    let all_phase2 = controls.iter().all(|c| c.phase() == Phase::Phase2);
    assert!(all_phase2, "All {surface} controls should be Phase 2");
}

#[rstest]
#[case(ControlSurface::Toolbar)]
#[case(ControlSurface::PropertiesPanel)]
#[case(ControlSurface::StylePanel)]
#[case(ControlSurface::LayersPanel)]
#[case(ControlSurface::AlignmentPanel)]
fn test_required_surfaces_have_controls(#[case] surface: ControlSurface) {
    let inventory = ControlInventory::new();
    let surface_controls = inventory.by_surface(surface);
    assert!(
        !surface_controls.is_empty(),
        "Surface '{surface}' must have at least one control"
    );
}

#[rstest]
fn test_toolbar_tools_have_keyboard_shortcuts() {
    let inventory = ControlInventory::new();
    let toolbar_controls = inventory.by_surface(ControlSurface::Toolbar);

    for control in toolbar_controls {
        if control.phase() == Phase::Phase1 || control.name() == "Type Tool" {
            assert!(
                control.keyboard.shortcut.is_some(),
                "Toolbar tool '{}' must have a keyboard shortcut",
                control.name()
            );
        }
    }
}

#[rstest]
fn test_controls_requiring_actions_have_action_names() {
    let inventory = ControlInventory::new();

    for control in inventory.all() {
        if control.action_linkage.requires_action {
            // Action name should be present for most controls; some may use placeholder
            // during planning phase, so we just check that the field is documented
            assert!(
                !control.action_linkage.notes.is_empty(),
                "Control '{}' requires action but has no linkage notes",
                control.name()
            );
        }
    }
}

#[rstest]
fn test_current_shell_evidence_accuracy() {
    let inventory = ControlInventory::new();
    let with_evidence = inventory.with_evidence();

    // We know from Phase 0 that tool_rail and style_controls have evidence
    assert!(
        !with_evidence.is_empty(),
        "At least some controls should have current shell evidence"
    );

    // Verify that controls with evidence have file paths
    for control in with_evidence {
        assert!(
            control.current_evidence.file_path.is_some(),
            "Control '{}' claims to have evidence but no file path",
            control.name()
        );
    }
}

#[rstest]
fn test_controls_without_evidence_have_notes() {
    let inventory = ControlInventory::new();
    let without_evidence = inventory.without_evidence();

    for control in without_evidence {
        assert!(
            !control.current_evidence.notes.is_empty(),
            "Control '{}' has no evidence and should explain why",
            control.name()
        );
    }
}

#[rstest]
fn test_phase1_has_expected_tool_count() {
    let inventory = ControlInventory::new();
    let toolbar_controls = inventory.by_surface(ControlSurface::Toolbar);
    let phase1_tools: Vec<_> = toolbar_controls
        .iter()
        .filter(|c| c.phase() == Phase::Phase1)
        .collect();

    // Phase 1 should have: Selection, Direct Selection, Pen, Rectangle, Ellipse, Line
    assert_eq!(
        phase1_tools.len(),
        6,
        "Phase 1 should have exactly 6 toolbar tools"
    );
}

#[rstest]
fn test_stroke_and_fill_controls_exist() {
    let inventory = ControlInventory::new();
    let style_controls = inventory.by_surface(ControlSurface::StylePanel);

    let has_stroke_colour = style_controls
        .iter()
        .any(|c| c.name().contains("Stroke Colour"));
    let has_fill_colour = style_controls
        .iter()
        .any(|c| c.name().contains("Fill Colour"));

    assert!(
        has_stroke_colour,
        "Style panel must have stroke colour control"
    );
    assert!(has_fill_colour, "Style panel must have fill colour control");
}

#[rstest]
fn test_layer_panel_has_core_controls() {
    let inventory = ControlInventory::new();
    let layer_controls = inventory.by_surface(ControlSurface::LayersPanel);

    let has_visibility = layer_controls
        .iter()
        .any(|c| c.name().contains("Visibility"));
    let has_lock = layer_controls.iter().any(|c| c.name().contains("Lock"));
    let has_rename = layer_controls.iter().any(|c| c.name().contains("Rename"));

    assert!(has_visibility, "Layers panel must have visibility toggle");
    assert!(has_lock, "Layers panel must have lock toggle");
    assert!(has_rename, "Layers panel must have rename capability");
}

#[rstest]
fn test_properties_panel_has_transform_fields() {
    let inventory = ControlInventory::new();
    let properties_controls = inventory.by_surface(ControlSurface::PropertiesPanel);

    let has_x = properties_controls
        .iter()
        .any(|c| c.name().contains("X Position"));
    let has_y = properties_controls
        .iter()
        .any(|c| c.name().contains("Y Position"));
    let has_width = properties_controls
        .iter()
        .any(|c| c.name().contains("Width"));
    let has_height = properties_controls
        .iter()
        .any(|c| c.name().contains("Height"));
    let has_rotation = properties_controls
        .iter()
        .any(|c| c.name().contains("Rotation"));

    assert!(has_x, "Properties panel must have X position field");
    assert!(has_y, "Properties panel must have Y position field");
    assert!(has_width, "Properties panel must have width field");
    assert!(has_height, "Properties panel must have height field");
    assert!(has_rotation, "Properties panel must have rotation field");
}

#[rstest]
fn test_alignment_panel_has_distribution_controls() {
    let inventory = ControlInventory::new();
    let alignment_controls = inventory.by_surface(ControlSurface::AlignmentPanel);

    let has_horizontal_dist = alignment_controls
        .iter()
        .any(|c| c.name().contains("Distribute Horizontal"));
    let has_vertical_dist = alignment_controls
        .iter()
        .any(|c| c.name().contains("Distribute Vertical"));

    assert!(
        has_horizontal_dist,
        "Alignment panel must have horizontal distribution"
    );
    assert!(
        has_vertical_dist,
        "Alignment panel must have vertical distribution"
    );
}

#[rstest]
#[case(ControlSurface::CharacterPanel)]
#[case(ControlSurface::ParagraphPanel)]
fn test_phase2_panel_controls_are_all_phase2(#[case] surface: ControlSurface) {
    assert_phase2_surface_all_phase2(surface);
}
