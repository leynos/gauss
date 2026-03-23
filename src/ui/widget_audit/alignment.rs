//! Alignment panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

pub(super) fn controls() -> Vec<RequiredControl> {
    let mut v = alignment_buttons();
    v.extend(distribution_buttons());
    v
}

fn alignment_buttons() -> Vec<RequiredControl> {
    vec![
        make_alignment_panel_button(
            "Align Left",
            "Align selected objects to the leftmost edge",
            AlignmentAction {
                name: "AlignObjectsLeft",
                notes: "Alignment operation must be undoable",
            },
            "1.3.4",
        ),
        make_alignment_panel_button(
            "Align Center Horizontal",
            "Align selected objects to horizontal center",
            AlignmentAction {
                name: "AlignObjectsCenterHorizontal",
                notes: "Alignment operation must be undoable",
            },
            "1.3.4",
        ),
        make_alignment_panel_button(
            "Align Right",
            "Align selected objects to the rightmost edge",
            AlignmentAction {
                name: "AlignObjectsRight",
                notes: "Alignment operation must be undoable",
            },
            "1.3.4",
        ),
        make_alignment_panel_button(
            "Align Top",
            "Align selected objects to the topmost edge",
            AlignmentAction {
                name: "AlignObjectsTop",
                notes: "Alignment operation must be undoable",
            },
            "1.3.4",
        ),
        make_alignment_panel_button(
            "Align Center Vertical",
            "Align selected objects to vertical center",
            AlignmentAction {
                name: "AlignObjectsCenterVertical",
                notes: "Alignment operation must be undoable",
            },
            "1.3.4",
        ),
        make_alignment_panel_button(
            "Align Bottom",
            "Align selected objects to the bottommost edge",
            AlignmentAction {
                name: "AlignObjectsBottom",
                notes: "Alignment operation must be undoable",
            },
            "1.3.4",
        ),
    ]
}

fn distribution_buttons() -> Vec<RequiredControl> {
    vec![
        make_alignment_panel_button(
            "Distribute Horizontal",
            "Distribute selected objects evenly along horizontal axis",
            AlignmentAction {
                name: "DistributeObjectsHorizontal",
                notes: "Distribution operation must be undoable",
            },
            "1.3.5",
        ),
        make_alignment_panel_button(
            "Distribute Vertical",
            "Distribute selected objects evenly along vertical axis",
            AlignmentAction {
                name: "DistributeObjectsVertical",
                notes: "Distribution operation must be undoable",
            },
            "1.3.5",
        ),
    ]
}

#[derive(Clone, Copy)]
struct AlignmentAction {
    name: &'static str,
    notes: &'static str,
}

fn make_alignment_panel_button(
    name: &'static str,
    description: &'static str,
    action: AlignmentAction,
    roadmap_ref: &'static str,
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
            action_name: Some(action.name),
            notes: action.notes,
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
