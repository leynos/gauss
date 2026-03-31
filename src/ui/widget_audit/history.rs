//! History panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

/// Returns the history panel control inventory.
///
/// Provides the history entry row control for displaying and selecting
/// history states for undo/redo operations.
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![RequiredControl {
        name: "History Entry Row",
        phase: Phase::Phase1,
        surface: ControlSurface::HistoryPanel,
        user_job: UserJob {
            description: "Display and select a history state for undo/redo",
        },
        states: ControlStates {
            states: vec!["current", "past", "future", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: None,
            keyboard_only_operation: true,
            notes: "Must support arrow keys for navigation, Enter to jump to state",
        },
        accessibility: AccessibilityRequirements {
            role: "ListItem",
            label: "History Entry",
            states: vec!["focusable", "selected"],
            notes: "Must announce action description and temporal position",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("JumpToHistoryState"),
            notes: "Jumping to history state routes through undo/redo system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.6.2"),
            RequirementSource::FeaturePlan("Phase 1: History UI"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "History panel UI not yet implemented",
        },
    }]
}
