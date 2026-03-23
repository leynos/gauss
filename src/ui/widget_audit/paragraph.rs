//! Paragraph panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        paragraph_spacing_field(),
        line_spacing_field(),
        indentation_controls(),
    ]
}

fn paragraph_spacing_field() -> RequiredControl {
    RequiredControl {
        name: "Paragraph Spacing Field",
        phase: Phase::Phase2,
        surface: ControlSurface::ParagraphPanel,
        user_job: UserJob {
            description: "Set spacing before/after paragraphs",
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
            label: "Paragraph Spacing",
            states: vec!["focusable", "editable"],
            notes: "Must announce current spacing value and units",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("SetParagraphSpacing"),
            notes: "Spacing change must be undoable",
        },
        sources: vec![
            RequirementSource::Roadmap("2.2"),
            RequirementSource::FeaturePlan("Phase 2: Paragraph Panel"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Phase 2 feature",
        },
    }
}

fn line_spacing_field() -> RequiredControl {
    RequiredControl {
        name: "Line Spacing Field",
        phase: Phase::Phase2,
        surface: ControlSurface::ParagraphPanel,
        user_job: UserJob {
            description: "Set leading (line height) for text",
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
            label: "Line Spacing",
            states: vec!["focusable", "editable"],
            notes: "Must announce current line height value",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("SetLineSpacing"),
            notes: "Line spacing change must be undoable",
        },
        sources: vec![
            RequirementSource::Roadmap("2.2"),
            RequirementSource::FeaturePlan("Phase 2: Paragraph Panel"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Phase 2 feature",
        },
    }
}

fn indentation_controls() -> RequiredControl {
    RequiredControl {
        name: "Indentation Controls",
        phase: Phase::Phase2,
        surface: ControlSurface::ParagraphPanel,
        user_job: UserJob {
            description: "Set left indent, right indent, and first-line indent",
        },
        states: ControlStates {
            states: vec!["enabled", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: None,
            keyboard_only_operation: true,
            notes: "Must support keyboard input for each indent field",
        },
        accessibility: AccessibilityRequirements {
            role: "TextInput",
            label: "Indentation",
            states: vec!["focusable", "editable"],
            notes: "Must announce which indent type and current value",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("SetParagraphIndent"),
            notes: "Indentation change must be undoable",
        },
        sources: vec![
            RequirementSource::Roadmap("2.2"),
            RequirementSource::FeaturePlan("Phase 2: Paragraph Panel"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Phase 2 feature",
        },
    }
}
