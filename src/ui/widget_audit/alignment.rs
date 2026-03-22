//! Alignment panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        make_alignment_panel_button(
            "Align Left",
            "Align selected objects to the leftmost edge",
            "AlignObjectsLeft",
            "1.3.4",
            "Alignment operation must be undoable",
        ),
        make_alignment_panel_button(
            "Align Center Horizontal",
            "Align selected objects to horizontal center",
            "AlignObjectsCenterHorizontal",
            "1.3.4",
            "Alignment operation must be undoable",
        ),
        make_alignment_panel_button(
            "Align Right",
            "Align selected objects to the rightmost edge",
            "AlignObjectsRight",
            "1.3.4",
            "Alignment operation must be undoable",
        ),
        make_alignment_panel_button(
            "Align Top",
            "Align selected objects to the topmost edge",
            "AlignObjectsTop",
            "1.3.4",
            "Alignment operation must be undoable",
        ),
        make_alignment_panel_button(
            "Align Center Vertical",
            "Align selected objects to vertical center",
            "AlignObjectsCenterVertical",
            "1.3.4",
            "Alignment operation must be undoable",
        ),
        make_alignment_panel_button(
            "Align Bottom",
            "Align selected objects to the bottommost edge",
            "AlignObjectsBottom",
            "1.3.4",
            "Alignment operation must be undoable",
        ),
        make_alignment_panel_button(
            "Distribute Horizontal",
            "Distribute selected objects evenly along horizontal axis",
            "DistributeObjectsHorizontal",
            "1.3.5",
            "Distribution operation must be undoable",
        ),
        make_alignment_panel_button(
            "Distribute Vertical",
            "Distribute selected objects evenly along vertical axis",
            "DistributeObjectsVertical",
            "1.3.5",
            "Distribution operation must be undoable",
        ),
    ]
}

#[expect(
    clippy::too_many_arguments,
    reason = "Builder function for alignment panel buttons; five parameters capture exactly the varying fields"
)]
fn make_alignment_panel_button(
    name: &'static str,
    description: &'static str,
    action_name: &'static str,
    roadmap_ref: &'static str,
    operation_notes: &'static str,
) -> RequiredControl {
    RequiredControl {
        name,
        phase: Phase::Phase1,
        surface: ControlSurface::AlignmentPanel,
        user_job: UserJob { description },
        states: ControlStates {
            states: vec!["enabled", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: None,
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard when panel is focused",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: name,
            states: vec!["focusable"],
            notes: "Must announce action when activated",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some(action_name),
            notes: operation_notes,
        },
        sources: vec![
            RequirementSource::Roadmap(roadmap_ref),
            RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Not yet implemented",
        },
    }
}
