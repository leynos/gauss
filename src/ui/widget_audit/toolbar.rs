//! Toolbar control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

#[derive(Clone, Copy)]
struct ToolbarControlSpec {
    name: &'static str,
    phase: Phase,
    description: &'static str,
    shortcut: &'static str,
    a11y_notes: &'static str,
    action_name: &'static str,
    roadmap_ref: &'static str,
    feature_plan: &'static str,
    evidence_exists: bool,
    evidence_path: Option<&'static str>,
    evidence_notes: &'static str,
}

fn make_toolbar_tool(spec: ToolbarControlSpec) -> RequiredControl {
    RequiredControl {
        name: spec.name,
        phase: spec.phase,
        surface: ControlSurface::Toolbar,
        user_job: UserJob { description: spec.description },
        states: ControlStates {
            states: vec!["selected", "disabled", "focused"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some(spec.shortcut),
            keyboard_only_operation: true,
            notes: "Must be activatable via keyboard shortcut",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: spec.name,
            states: vec!["pressed", "focusable"],
            notes: spec.a11y_notes,
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some(spec.action_name),
            notes: "Tool activation routes through action system",
        },
        sources: vec![
            RequirementSource::Roadmap(spec.roadmap_ref),
            RequirementSource::FeaturePlan(spec.feature_plan),
        ],
        current_evidence: CurrentShellEvidence {
            exists: spec.evidence_exists,
            file_path: spec.evidence_path,
            notes: spec.evidence_notes,
        },
    }
}

/// Toolbar controls (7 tools).
pub(super) fn controls() -> Vec<RequiredControl> {
    TOOL_SPECS.iter().copied().map(make_toolbar_tool).collect()
}

const TOOL_SPECS: &[ToolbarControlSpec] = &[
    ToolbarControlSpec {
        name: "Selection Tool",
        phase: Phase::Phase1,
        description: "Select and move entire objects",
        shortcut: "V",
        a11y_notes: "Must announce tool activation",
        action_name: "ActivateSelectionTool",
        roadmap_ref: "1.2.2",
        feature_plan: "Phase 1: Selection Tools",
        evidence_exists: true,
        evidence_path: Some("src/ui/phase0_shell/tool_rail.rs"),
        evidence_notes: "Manipulate mode tool exists in current shell",
    },
    ToolbarControlSpec {
        name: "Direct Selection Tool",
        phase: Phase::Phase1,
        description: "Select and edit individual anchor points or path segments",
        shortcut: "A",
        a11y_notes: "Must announce tool activation and distinguish from Selection Tool",
        action_name: "ActivateDirectSelectionTool",
        roadmap_ref: "1.2.2",
        feature_plan: "Phase 1: Selection Tools",
        evidence_exists: true,
        evidence_path: Some("src/ui/phase0_shell/tool_rail.rs"),
        evidence_notes: "Manipulate mode with anchor/handle selection exists",
    },
    ToolbarControlSpec {
        name: "Pen Tool",
        phase: Phase::Phase1,
        description: "Create freeform paths with Bezier curves",
        shortcut: "P",
        a11y_notes: "Must announce tool activation",
        action_name: "ActivatePenTool",
        roadmap_ref: "1.1.4",
        feature_plan: "Phase 1: Basic Shape Drawing",
        evidence_exists: true,
        evidence_path: Some("src/ui/phase0_shell/tool_rail.rs"),
        evidence_notes: "Draw mode tool exists in current shell",
    },
    ToolbarControlSpec {
        name: "Rectangle Tool",
        phase: Phase::Phase1,
        description: "Create rectangles and squares",
        shortcut: "R",
        a11y_notes: "Must announce tool activation",
        action_name: "ActivateRectangleTool",
        roadmap_ref: "1.1.1",
        feature_plan: "Phase 1: Basic Shape Drawing",
        evidence_exists: false,
        evidence_path: None,
        evidence_notes: "Not yet implemented",
    },
    ToolbarControlSpec {
        name: "Ellipse Tool",
        phase: Phase::Phase1,
        description: "Create ellipses and circles",
        shortcut: "O",
        a11y_notes: "Must announce tool activation",
        action_name: "ActivateEllipseTool",
        roadmap_ref: "1.1.2",
        feature_plan: "Phase 1: Basic Shape Drawing",
        evidence_exists: false,
        evidence_path: None,
        evidence_notes: "Not yet implemented",
    },
    ToolbarControlSpec {
        name: "Line Tool",
        phase: Phase::Phase1,
        description: "Create straight line segments",
        shortcut: "\\",
        a11y_notes: "Must announce tool activation",
        action_name: "ActivateLineTool",
        roadmap_ref: "1.1.3",
        feature_plan: "Phase 1: Basic Shape Drawing",
        evidence_exists: false,
        evidence_path: None,
        evidence_notes: "Not yet implemented",
    },
    ToolbarControlSpec {
        name: "Type Tool",
        phase: Phase::Phase2,
        description: "Create and edit text objects",
        shortcut: "T",
        a11y_notes: "Must announce tool activation and text editing mode",
        action_name: "ActivateTypeTool",
        roadmap_ref: "2.1.1",
        feature_plan: "Phase 2: Text Engine",
        evidence_exists: false,
        evidence_path: None,
        evidence_notes: "Phase 2 feature",
    },
];
