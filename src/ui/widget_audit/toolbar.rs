//! Toolbar control definitions.
//!
//! This module defines the toolbar tool controls required for Phase 1 and Phase 2,
//! including:
//! - **Selection Tool** (`V`) — Select and move entire objects
//! - **Direct Selection Tool** (`A`) — Select and edit anchor points or path segments
//! - **Pen Tool** (`P`) — Create freeform paths with Bezier curves
//! - **Rectangle Tool** (`R`) — Create rectangles and squares
//! - **Ellipse Tool** (`O`) — Create ellipses and circles
//! - **Line Tool** (`\`) — Create straight line segments
//! - **Type Tool** (`T`) — Create and edit text objects (Phase 2)
//!
//! These controls are used by the widget capability audit to verify that
//! the Phase 0 shell implements the required toolbar functionality.
//! Each tool specifies its own states via `ControlStates`, allowing tools
//! like the Type Tool to define custom state lists as needed.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

/// Specification for a toolbar tool control.
///
/// Each tool defines its own states via the `states` field, allowing
/// customization for tools with unique state requirements.
#[derive(Clone, Copy)]
struct ToolbarControlSpec {
    /// Human-readable control name, also used as its accessibility label.
    name: &'static str,
    /// Roadmap phase in which the control is required.
    phase: Phase,
    /// User task supported by the control.
    description: &'static str,
    /// Keyboard shortcut used to activate the control.
    shortcut: &'static str,
    /// State names the control must expose.
    states: &'static [&'static str],
    /// Accessibility announcement and state requirements.
    a11y_notes: &'static str,
    /// Action-system name associated with activating the control.
    action_name: &'static str,
    /// Identifier of the roadmap requirement for this control.
    roadmap_ref: &'static str,
    /// Feature-plan source that details the requirement.
    feature_plan: &'static str,
    /// Whether current shell implementation evidence exists.
    evidence_exists: bool,
    /// Location of the evidence when one is available.
    evidence_path: Option<&'static str>,
    /// Qualification or absence note for the current shell evidence.
    evidence_notes: &'static str,
}

/// Convert a toolbar specification into its audit record.
/// For example, Selection Tool produces a toolbar record with shortcut `V` and
/// action `ActivateSelectionTool`.
fn make_toolbar_tool(spec: ToolbarControlSpec) -> RequiredControl {
    RequiredControl {
        name: spec.name,
        phase: spec.phase,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: spec.description,
        },
        states: ControlStates {
            states: spec.states.to_vec(),
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

/// Standard tool states used by most toolbar tools.
const STANDARD_TOOL_STATES: &[&str] = &["selected", "disabled", "focused"];

/// Toolbar controls (7 tools).
pub(super) fn controls() -> Vec<RequiredControl> {
    TOOL_SPECS.iter().copied().map(make_toolbar_tool).collect()
}

/// Specifications for the seven Phase 1 and Phase 2 toolbar tools.
const TOOL_SPECS: &[ToolbarControlSpec] = &[
    ToolbarControlSpec {
        name: "Selection Tool",
        phase: Phase::Phase1,
        description: "Select and move entire objects",
        shortcut: "V",
        states: STANDARD_TOOL_STATES,
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
        states: STANDARD_TOOL_STATES,
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
        states: STANDARD_TOOL_STATES,
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
        states: STANDARD_TOOL_STATES,
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
        states: STANDARD_TOOL_STATES,
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
        states: STANDARD_TOOL_STATES,
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
        states: STANDARD_TOOL_STATES,
        a11y_notes: "Must announce tool activation and text editing mode",
        action_name: "ActivateTypeTool",
        roadmap_ref: "2.1.1",
        feature_plan: "Phase 2: Text Engine",
        evidence_exists: false,
        evidence_path: None,
        evidence_notes: "Phase 2 feature",
    },
];
