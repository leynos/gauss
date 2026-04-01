//! Properties panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};
use gauss_core::model::Action;

#[derive(Clone, Copy)]
struct A11ySpec {
    label: &'static str,
    notes: &'static str,
}

fn make_numeric_field(
    name: &'static str,
    description: &'static str,
    a11y: A11ySpec,
    action_name: Action,
) -> RequiredControl {
    RequiredControl {
        name,
        phase: Phase::Phase1,
        surface: ControlSurface::PropertiesPanel,
        user_job: UserJob { description },
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
            label: a11y.label,
            states: vec!["focusable", "editable", "read-only"],
            notes: a11y.notes,
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some(action_name.identifier()),
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
    }
}

/// Returns the properties panel control inventory.
///
/// Provides numeric field controls for transform properties (position,
/// dimensions, rotation) of selected objects.
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        make_numeric_field(
            "X Position Field",
            "View and edit object X coordinate",
            A11ySpec {
                label: "X Position",
                notes: "Must announce current value and accept numeric input",
            },
            Action::SetObjectPosition,
        ),
        make_numeric_field(
            "Y Position Field",
            "View and edit object Y coordinate",
            A11ySpec {
                label: "Y Position",
                notes: "Must announce current value and accept numeric input",
            },
            Action::SetObjectPosition,
        ),
        make_numeric_field(
            "Width Field",
            "View and edit object width",
            A11ySpec {
                label: "Width",
                notes: "Must announce current value and accept numeric input",
            },
            Action::SetObjectSize,
        ),
        make_numeric_field(
            "Height Field",
            "View and edit object height",
            A11ySpec {
                label: "Height",
                notes: "Must announce current value and accept numeric input",
            },
            Action::SetObjectSize,
        ),
        make_numeric_field(
            "Rotation Field",
            "View and edit object rotation angle",
            A11ySpec {
                label: "Rotation",
                notes: "Must announce current value in degrees and accept numeric input",
            },
            Action::SetObjectRotation,
        ),
    ]
}
