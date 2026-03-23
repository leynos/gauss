//! Toolbar control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

/// Toolbar controls (7 tools).
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        selection_tool(),
        direct_selection_tool(),
        pen_tool(),
        rectangle_tool(),
        ellipse_tool(),
        line_tool(),
        type_tool(),
    ]
}

fn selection_tool() -> RequiredControl {
    RequiredControl {
        name: "Selection Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Select and move entire objects",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("V"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Selection Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivateSelectionTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.2.2"),
            RequirementSource::FeaturePlan("Phase 1: Selection Tools"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: true,
            file_path: Some("src/ui/phase0_shell/tool_rail.rs"),
            notes: "Manipulate mode tool exists in current shell",
        },
    }
}

fn direct_selection_tool() -> RequiredControl {
    RequiredControl {
        name: "Direct Selection Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Select and edit individual anchor points or path segments",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("A"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Direct Selection Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation and distinguish from Selection Tool",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivateDirectSelectionTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.2.2"),
            RequirementSource::FeaturePlan("Phase 1: Selection Tools"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: true,
            file_path: Some("src/ui/phase0_shell/tool_rail.rs"),
            notes: "Manipulate mode with anchor/handle selection exists",
        },
    }
}

fn pen_tool() -> RequiredControl {
    RequiredControl {
        name: "Pen Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Create freeform paths with Bezier curves",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("P"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Pen Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivatePenTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.1.4"),
            RequirementSource::FeaturePlan("Phase 1: Basic Shape Drawing"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: true,
            file_path: Some("src/ui/phase0_shell/tool_rail.rs"),
            notes: "Draw mode tool exists in current shell",
        },
    }
}

fn rectangle_tool() -> RequiredControl {
    RequiredControl {
        name: "Rectangle Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Create rectangles and squares",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("R"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Rectangle Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivateRectangleTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.1.1"),
            RequirementSource::FeaturePlan("Phase 1: Basic Shape Drawing"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Not yet implemented",
        },
    }
}

fn ellipse_tool() -> RequiredControl {
    RequiredControl {
        name: "Ellipse Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Create ellipses and circles",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("O"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Ellipse Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivateEllipseTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.1.2"),
            RequirementSource::FeaturePlan("Phase 1: Basic Shape Drawing"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Not yet implemented",
        },
    }
}

fn line_tool() -> RequiredControl {
    RequiredControl {
        name: "Line Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Create straight line segments",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("\\"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Line Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivateLineTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("1.1.3"),
            RequirementSource::FeaturePlan("Phase 1: Basic Shape Drawing"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Not yet implemented",
        },
    }
}

fn type_tool() -> RequiredControl {
    RequiredControl {
        name: "Type Tool",
        phase: Phase::Phase2,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Create and edit text objects",
        },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("T"),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "Type Tool",
            states: vec!["pressed", "focusable"],
            notes: "Must announce tool activation and text editing mode",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some("ActivateTypeTool"),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap("2.1.1"),
            RequirementSource::FeaturePlan("Phase 2: Text Engine"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Phase 2 feature",
        },
    }
}
