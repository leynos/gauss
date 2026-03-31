//! Canvas text editor control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

/// Returns the canvas text editor control inventory.
///
/// Provides the inline text cursor control for positioning and editing text
/// directly on the canvas.
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![RequiredControl {
        name: "Inline Text Cursor",
        phase: Phase::Phase2,
        surface: ControlSurface::CanvasTextEditor,
        user_job: UserJob {
            description: "Position cursor within text for editing",
        },
        states: ControlStates {
            states: vec!["visible", "blinking", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: None,
            keyboard_only_operation: true,
            notes: "Must support arrow keys, Home/End, text selection shortcuts",
        },
        accessibility: AccessibilityRequirements {
            role: "TextInput",
            label: "Text Editor",
            states: vec!["focusable", "editable"],
            notes: "Must announce cursor position and text content",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("EditText"),
            notes: "Text edits route through command system for undo/redo",
        },
        sources: vec![
            RequirementSource::Roadmap("2.1.2"),
            RequirementSource::FeaturePlan("Phase 2: Text Engine"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Phase 2 feature",
        },
    }]
}
