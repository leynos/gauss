//! Properties panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

#[expect(
    clippy::too_many_lines,
    reason = "Flat data table for controls; no meaningful sub-structure to extract"
)]
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        RequiredControl {
            name: "X Position Field",
            phase: Phase::Phase1,
            surface: ControlSurface::PropertiesPanel,
            user_job: UserJob {
                description: "View and edit object X coordinate",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "read-only"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard input, arrow keys for nudge, Enter to commit",
            },
            accessibility: AccessibilityRequirements {
                role: "TextInput",
                label: "X Position",
                states: vec!["focusable", "editable"],
                notes: "Must announce current value and accept numeric input",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetObjectPosition"),
                notes: "Value change emits command for undo/redo",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.3"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Y Position Field",
            phase: Phase::Phase1,
            surface: ControlSurface::PropertiesPanel,
            user_job: UserJob {
                description: "View and edit object Y coordinate",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "read-only"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard input, arrow keys for nudge, Enter to commit",
            },
            accessibility: AccessibilityRequirements {
                role: "TextInput",
                label: "Y Position",
                states: vec!["focusable", "editable"],
                notes: "Must announce current value and accept numeric input",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetObjectPosition"),
                notes: "Value change emits command for undo/redo",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.3"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Width Field",
            phase: Phase::Phase1,
            surface: ControlSurface::PropertiesPanel,
            user_job: UserJob {
                description: "View and edit object width",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "read-only"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard input, arrow keys for nudge, Enter to commit",
            },
            accessibility: AccessibilityRequirements {
                role: "TextInput",
                label: "Width",
                states: vec!["focusable", "editable"],
                notes: "Must announce current value and accept numeric input",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetObjectSize"),
                notes: "Value change emits command for undo/redo",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.3"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Height Field",
            phase: Phase::Phase1,
            surface: ControlSurface::PropertiesPanel,
            user_job: UserJob {
                description: "View and edit object height",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "read-only"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard input, arrow keys for nudge, Enter to commit",
            },
            accessibility: AccessibilityRequirements {
                role: "TextInput",
                label: "Height",
                states: vec!["focusable", "editable"],
                notes: "Must announce current value and accept numeric input",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetObjectSize"),
                notes: "Value change emits command for undo/redo",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.3"),
                RequirementSource::FeaturePlan("Phase 1: Transformation & Alignment"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Not yet implemented",
            },
        },
        RequiredControl {
            name: "Rotation Field",
            phase: Phase::Phase1,
            surface: ControlSurface::PropertiesPanel,
            user_job: UserJob {
                description: "View and edit object rotation angle",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "read-only"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard input, arrow keys for nudge, Enter to commit",
            },
            accessibility: AccessibilityRequirements {
                role: "TextInput",
                label: "Rotation",
                states: vec!["focusable", "editable"],
                notes: "Must announce current value in degrees and accept numeric input",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetObjectRotation"),
                notes: "Value change emits command for undo/redo",
            },
            sources: vec![
                RequirementSource::Roadmap("1.3.3"),
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
