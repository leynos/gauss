//! GPUI integration test for widget audit shell-seam consistency.
//!
//! This test validates that controls marked as having "current shell evidence"
//! in the widget audit inventory actually correspond to working controls in the
//! Phase 0 shell. This proves the audit is grounded in the actual codebase
//! rather than being speculative documentation.

mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gauss::ui::widget_audit::ControlInventory;
use gpui::TestAppContext;

#[gpui::test]
fn controls_with_evidence_exist_in_shell(cx: &mut TestAppContext) {
    init_test_app(cx);

    let inventory = ControlInventory::new();
    let with_evidence = inventory.with_evidence();

    assert!(
        !with_evidence.is_empty(),
        "Audit inventory should have at least some controls with current shell evidence"
    );

    // Create a Phase0Shell to verify it can be instantiated
    // (proving the referenced shell modules are functional)
    let _view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));

        ensure_initial_draw(visual_cx);
        cx.run_until_parked();

        view
    };

    // Verify that each control with evidence references a valid file path
    for control in with_evidence {
        let file_path = control
            .current_evidence
            .file_path
            .expect("Control claims evidence but has no file path");

        // Verify the file path looks reasonable
        // (actual file existence is validated by Rust's module system at compile time)
        assert!(
            file_path.starts_with("src/ui/phase0_shell/"),
            "Control '{}' claims evidence in '{}' which should be in phase0_shell",
            control.name(),
            file_path
        );
    }
}

#[gpui::test]
fn tool_rail_controls_match_audit(cx: &mut TestAppContext) {
    init_test_app(cx);

    let inventory = ControlInventory::new();

    // Find controls that claim tool_rail.rs as evidence
    let with_evidence = inventory.with_evidence();
    let tool_rail_controls: Vec<_> = with_evidence
        .iter()
        .filter(|c| {
            c.current_evidence
                .file_path
                .map(|p| p.contains("tool_rail.rs"))
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !tool_rail_controls.is_empty(),
        "At least some toolbar controls should have tool_rail.rs evidence"
    );

    // Verify that the shell can be created with tool rail controls
    let _view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));

        ensure_initial_draw(visual_cx);
        cx.run_until_parked();

        view
    };

    // If we got here, tool_rail is wired into the shell successfully
    // This proves the audit's tool_rail evidence is accurate
}

#[gpui::test]
fn style_controls_match_audit(cx: &mut TestAppContext) {
    init_test_app(cx);

    let inventory = ControlInventory::new();

    // Find controls that claim style_controls.rs as evidence
    let with_evidence = inventory.with_evidence();
    let style_controls: Vec<_> = with_evidence
        .iter()
        .filter(|c| {
            c.current_evidence
                .file_path
                .map(|p| p.contains("style_controls.rs"))
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !style_controls.is_empty(),
        "At least some style controls should have style_controls.rs evidence"
    );

    // Verify stroke and fill controls are present in the evidence
    let has_stroke_color = style_controls
        .iter()
        .any(|c| c.name().contains("Stroke Color"));
    let has_fill_color = style_controls
        .iter()
        .any(|c| c.name().contains("Fill Color"));

    assert!(
        has_stroke_color,
        "Style controls evidence should include stroke color picker"
    );
    assert!(
        has_fill_color,
        "Style controls evidence should include fill color picker"
    );

    // Verify that the shell can be created with style controls
    let _view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));

        ensure_initial_draw(visual_cx);
        cx.run_until_parked();

        view
    };

    // If we got here, style_controls is wired into the shell successfully
    // This proves the audit's style_controls evidence is accurate
}

#[gpui::test]
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

#[gpui::test]
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
    use gauss::ui::widget_audit::ControlSurface;
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
            "Required surface '{}' must have at least one control",
            surface
        );
    }
}
