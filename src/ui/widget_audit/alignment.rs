//! Alignment panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource,
    UserJob,
};

#[expect(
    clippy::too_many_lines,
    reason = "Flat data table for controls; no meaningful sub-structure to extract"
)]
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        RequiredControl {
            name: "Align Left",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Align selected objects to the leftmost edge",
            },
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
                label: "Align Left",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("AlignObjectsLeft"),
                notes: "Alignment operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.4"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Align Center Horizontal",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Align selected objects to horizontal center",
            },
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
                label: "Align Center Horizontal",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("AlignObjectsCenterHorizontal"),
                notes: "Alignment operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.4"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Align Right",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Align selected objects to the rightmost edge",
            },
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
                label: "Align Right",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("AlignObjectsRight"),
                notes: "Alignment operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.4"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Align Top",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Align selected objects to the topmost edge",
            },
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
                label: "Align Top",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("AlignObjectsTop"),
                notes: "Alignment operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.4"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Align Center Vertical",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Align selected objects to vertical center",
            },
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
                label: "Align Center Vertical",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("AlignObjectsCenterVertical"),
                notes: "Alignment operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.4"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Align Bottom",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Align selected objects to the bottommost edge",
            },
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
                label: "Align Bottom",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("AlignObjectsBottom"),
                notes: "Alignment operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.4"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Distribute Horizontal",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Distribute selected objects evenly along horizontal axis",
            },
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
                label: "Distribute Horizontal",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("DistributeObjectsHorizontal"),
                notes: "Distribution operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.5"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Distribute Vertical",
            phase: Phase::Phase1,
            surface: ControlSurface::AlignmentPanel,
            user_job: UserJob {
                description: "Distribute selected objects evenly along vertical axis",
            },
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
                label: "Distribute Vertical",
                states: vec!["focusable"],
                notes: "Must announce action when activated",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("DistributeObjectsVertical"),
                notes: "Distribution operation must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.5"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
    ]
}
