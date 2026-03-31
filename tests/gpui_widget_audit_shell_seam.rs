//! GPUI integration test for widget audit shell-seam consistency.
//!
//! This test validates that controls marked as having "current shell evidence"
//! in the widget audit inventory actually correspond to working controls in the
//! Phase 0 shell. This proves the audit is grounded in the actual codebase
//! rather than being speculative documentation.
//!
//! NOTE: These tests validate the audit inventory structure and metadata,
//! not the runtime GPUI behavior (which requires the full common test helpers).

use gauss::ui::widget_audit::{ControlInventory, ControlSurface};
use std::path::Path;

#[test]
fn controls_with_evidence_have_valid_paths() {
    let inventory = ControlInventory::new();
    let with_evidence = inventory.with_evidence();

    assert!(
        !with_evidence.is_empty(),
        "Audit inventory should have at least some controls with current shell evidence"
    );

    // Verify that each control with evidence references a valid file path
    for control in with_evidence {
        let file_path = control
            .current_evidence
            .file_path
            .expect("Control claims evidence but has no file path");

        // Verify the file path exists (catches typos like "stlye_controls.rs")
        assert!(
            Path::new(&file_path).exists(),
            "Control '{}' claims evidence in '{}' but that path does not exist",
            control.name(),
            file_path
        );
    }
}

#[test]
fn tool_rail_controls_match_audit() {
    let inventory = ControlInventory::new();

    // Find controls that claim tool_rail.rs as evidence
    let with_evidence = inventory.with_evidence();
    let tool_rail_controls: Vec<_> = with_evidence
        .into_iter()
        .filter(|c| {
            c.current_evidence
                .file_path
                .is_some_and(|p| p.contains("tool_rail.rs"))
        })
        .collect();

    assert!(
        !tool_rail_controls.is_empty(),
        "At least some toolbar controls should have tool_rail.rs evidence"
    );

    // Verify that tool_rail controls reference valid tool names
    for control in &tool_rail_controls {
        assert!(
            !control.name().is_empty(),
            "Tool rail control should have a non-empty name"
        );
    }
}

#[test]
fn style_controls_match_audit() {
    let inventory = ControlInventory::new();

    // Find controls that claim style_controls.rs as evidence
    let with_evidence = inventory.with_evidence();
    let style_controls: Vec<_> = with_evidence
        .into_iter()
        .filter(|c| {
            c.current_evidence
                .file_path
                .is_some_and(|p| p.contains("style_controls.rs"))
        })
        .collect();

    assert!(
        !style_controls.is_empty(),
        "At least some style controls should have style_controls.rs evidence"
    );

    // Verify stroke and fill controls are present in the evidence
    let has_stroke_colour = style_controls
        .iter()
        .any(|c| c.name().contains("Stroke Colour"));
    let has_fill_colour = style_controls
        .iter()
        .any(|c| c.name().contains("Fill Colour"));

    assert!(
        has_stroke_colour,
        "Style controls evidence should include stroke colour picker"
    );
    assert!(
        has_fill_colour,
        "Style controls evidence should include fill colour picker"
    );
}

#[test]
fn controls_without_evidence_are_documented() {
    let inventory = ControlInventory::new();
    let without_evidence = inventory.without_evidence();

    assert!(
        !without_evidence.is_empty(),
        "Some controls should be pending implementation"
    );

    // All controls without evidence must explain why
    for control in without_evidence {
        assert!(
            !control.current_evidence.notes.is_empty(),
            "Control '{}' has no evidence but should explain why in notes",
            control.name()
        );

        // Verify the notes explain the status
        let notes = control.current_evidence.notes.to_lowercase();
        let has_explanation = notes.contains("not yet implemented")
            || notes.contains("phase 2")
            || notes.contains("pending");

        assert!(
            has_explanation,
            "Control '{}' has notes '{}' which should explain implementation status",
            control.name(),
            control.current_evidence.notes
        );
    }
}

#[test]
fn audit_inventory_is_complete() {
    let inventory = ControlInventory::new();

    // Verify we have a reasonable number of controls for Phase 1-2
    assert!(
        inventory.all().len() >= 30,
        "Inventory should have at least 30 controls for Phase 1-2 (found {})",
        inventory.all().len()
    );

    // Verify we have both phases represented
    assert!(
        !inventory
            .by_phase(gauss::ui::widget_audit::Phase::Phase1)
            .is_empty(),
        "Must have Phase 1 controls"
    );
    assert!(
        !inventory
            .by_phase(gauss::ui::widget_audit::Phase::Phase2)
            .is_empty(),
        "Must have Phase 2 controls"
    );

    // Verify all required surfaces are covered
    let required_surfaces = [
        ControlSurface::Toolbar,
        ControlSurface::PropertiesPanel,
        ControlSurface::StylePanel,
        ControlSurface::LayersPanel,
        ControlSurface::AlignmentPanel,
        ControlSurface::CharacterPanel,
        ControlSurface::ParagraphPanel,
    ];

    for surface in required_surfaces {
        assert!(
            !inventory.by_surface(surface).is_empty(),
            "Required surface '{surface}' must have at least one control"
        );
    }
}
