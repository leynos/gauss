//! Character panel control definitions.

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
            name: "Font Family Selector",
            phase: Phase::Phase2,
            surface: ControlSurface::CharacterPanel,
            user_job: UserJob {
                description: "Select font family for text",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "open", "closed"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard navigation and search through font list",
            },
            accessibility: AccessibilityRequirements {
                role: "ComboBox",
                label: "Font Family",
                states: vec!["focusable", "expanded", "collapsed"],
                notes: "Must announce current font family and selection changes",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetFontFamily"),
                notes: "Font change must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("2.2"),
                RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Phase 2 feature",
            },
        },
        RequiredControl {
            name: "Font Size Field",
            phase: Phase::Phase2,
            surface: ControlSurface::CharacterPanel,
            user_job: UserJob {
                description: "Set font size in points",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard input, arrow keys for nudge",
            },
            accessibility: AccessibilityRequirements {
                role: "TextInput",
                label: "Font Size",
                states: vec!["focusable", "editable"],
                notes: "Must announce current size in points",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetFontSize"),
                notes: "Size change must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("2.2"),
                RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Phase 2 feature",
            },
        },
        RequiredControl {
            name: "Bold Toggle",
            phase: Phase::Phase2,
            surface: ControlSurface::CharacterPanel,
            user_job: UserJob {
                description: "Apply or remove bold formatting",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "active"],
            },
            keyboard: KeyboardRequirements {
                shortcut: Some("Cmd+B / Ctrl+B"),
                keyboard_only_operation: true,
                notes: "Standard bold shortcut must work",
            },
            accessibility: AccessibilityRequirements {
                role: "ToggleButton",
                label: "Bold",
                states: vec!["focusable", "checked", "unchecked"],
                notes: "Must announce formatting state",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("ToggleBold"),
                notes: "Formatting change must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("2.2"),
                RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Phase 2 feature",
            },
        },
        RequiredControl {
            name: "Italic Toggle",
            phase: Phase::Phase2,
            surface: ControlSurface::CharacterPanel,
            user_job: UserJob {
                description: "Apply or remove italic formatting",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "active"],
            },
            keyboard: KeyboardRequirements {
                shortcut: Some("Cmd+I / Ctrl+I"),
                keyboard_only_operation: true,
                notes: "Standard italic shortcut must work",
            },
            accessibility: AccessibilityRequirements {
                role: "ToggleButton",
                label: "Italic",
                states: vec!["focusable", "checked", "unchecked"],
                notes: "Must announce formatting state",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("ToggleItalic"),
                notes: "Formatting change must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("2.2"),
                RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Phase 2 feature",
            },
        },
        RequiredControl {
            name: "Text Alignment Buttons",
            phase: Phase::Phase2,
            surface: ControlSurface::CharacterPanel,
            user_job: UserJob {
                description: "Set text alignment (left, center, right, justify)",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "selected"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard navigation between alignment options",
            },
            accessibility: AccessibilityRequirements {
                role: "RadioGroup",
                label: "Text Alignment",
                states: vec!["focusable"],
                notes: "Must announce current alignment selection",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetTextAlignment"),
                notes: "Alignment change must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("2.2"),
                RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Phase 2 feature",
            },
        },
        RequiredControl {
            name: "Text Color Picker",
            phase: Phase::Phase2,
            surface: ControlSurface::CharacterPanel,
            user_job: UserJob {
                description: "Set color for text characters",
            },
            states: ControlStates {
                states: vec!["enabled", "disabled", "focused", "open", "closed"],
            },
            keyboard: KeyboardRequirements {
                shortcut: None,
                keyboard_only_operation: true,
                notes: "Must support keyboard navigation through color picker",
            },
            accessibility: AccessibilityRequirements {
                role: "ColorPicker",
                label: "Text Color",
                states: vec!["focusable", "expanded", "collapsed"],
                notes: "Must announce current color value; reuses fill/stroke color infrastructure per roadmap 2.2",
            },
            action_linkage: ActionCommandLinkage {
                requires_action: true,
                action_name: Some("SetTextColor"),
                notes: "Color change must be undoable",
            },
            sources: vec![
                RequirementSource::Roadmap("2.2"),
                RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
            ],
            current_evidence: CurrentShellEvidence {
                exists: false,
                file_path: None,
                notes: "Phase 2 feature; will reuse color picker infrastructure",
            },
        },
    ]
}
