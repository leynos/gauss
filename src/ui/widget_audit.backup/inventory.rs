//! Control inventory implementation with builder functions.

use super::control::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, CurrentShellEvidence,
    KeyboardRequirements, RequiredControl, UserJob,
};
use super::types::{ControlSurface, Phase, RequirementSource};

/// Complete inventory of required controls for Phase 1-2.
pub struct ControlInventory {
    controls: Vec<RequiredControl>,
}
impl ControlInventory {
    /// Creates a new inventory with the complete Phase 1-2 control set.
    #[must_use]
    pub fn new() -> Self {
        let mut controls = Vec::new();
        controls.extend(Self::toolbar_controls());
        controls.extend(Self::properties_panel_controls());
        controls.extend(Self::alignment_panel_controls());
        controls.extend(Self::style_panel_stroke_controls());
        controls.extend(Self::style_panel_fill_controls());
        controls.extend(Self::layers_panel_controls());
        controls.extend(Self::history_panel_controls());
        controls.extend(Self::character_panel_controls());
        controls.extend(Self::paragraph_panel_controls());
        controls.extend(Self::canvas_text_editor_controls());
        Self { controls }
    }

    /// Toolbar controls (7 tools).
    #[expect(
        clippy::too_many_lines,
        reason = "Flat data table for Toolbar controls; no meaningful sub-structure to extract"
    )]
    fn toolbar_controls() -> Vec<RequiredControl> {
        vec![
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
            },
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
            },
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
            },
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
            },
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
            },
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
            },
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
            },
        ]
    }

    /// Properties Panel controls (5 fields).
    #[expect(
        clippy::too_many_lines,
        reason = "Flat data table for Properties Panel controls; no meaningful sub-structure to extract"
    )]
    fn properties_panel_controls() -> Vec<RequiredControl> {
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

    /// Alignment Panel controls (8 controls).
    #[expect(
        clippy::too_many_lines,
        reason = "Flat data table for Alignment Panel controls; no meaningful sub-structure to extract"
    )]
    fn alignment_panel_controls() -> Vec<RequiredControl> {
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

    /// Style Panel stroke controls (3 controls).
    fn style_panel_stroke_controls() -> Vec<RequiredControl> {
        vec![
            RequiredControl {
                name: "Stroke Color Picker",
                phase: Phase::Phase1,
                surface: ControlSurface::StylePanel,
                user_job: UserJob {
                    description: "Select stroke color for shapes",
                },
                states: ControlStates {
                    states: vec!["enabled", "disabled", "focused", "open", "closed"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support keyboard navigation through color picker UI",
                },
                accessibility: AccessibilityRequirements {
                    role: "ColorPicker",
                    label: "Stroke Color",
                    states: vec!["focusable", "expanded", "collapsed"],
                    notes: "Must announce current color value",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("SetStrokeColor"),
                    notes: "Color change emits command for undo/redo",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.4.1"),
                    RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                    RequirementSource::Architecture("14.1"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: true,
                    file_path: Some("src/ui/phase0_shell/style_controls.rs"),
                    notes: "Color picker for stroke exists in current shell",
                },
            },
            RequiredControl {
                name: "Stroke Width Field",
                phase: Phase::Phase1,
                surface: ControlSurface::StylePanel,
                user_job: UserJob {
                    description: "Set stroke width in pixels or points",
                },
                states: ControlStates {
                    states: vec!["enabled", "disabled", "focused", "read-only"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support keyboard input, arrow keys for nudge",
                },
                accessibility: AccessibilityRequirements {
                    role: "TextInput",
                    label: "Stroke Width",
                    states: vec!["focusable", "editable"],
                    notes: "Must announce current value and units",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("SetStrokeWidth"),
                    notes: "Width change emits command for undo/redo",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.4.1"),
                    RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: true,
                    file_path: Some("src/ui/phase0_shell/style_controls.rs"),
                    notes: "Stroke width control exists in current shell",
                },
            },
            RequiredControl {
                name: "Stroke Opacity Slider",
                phase: Phase::Phase1,
                surface: ControlSurface::StylePanel,
                user_job: UserJob {
                    description: "Adjust stroke transparency from 0% to 100%",
                },
                states: ControlStates {
                    states: vec!["enabled", "disabled", "focused"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support arrow keys for value adjustment",
                },
                accessibility: AccessibilityRequirements {
                    role: "Slider",
                    label: "Stroke Opacity",
                    states: vec!["focusable"],
                    notes: "Must announce current percentage value",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("SetStrokeOpacity"),
                    notes: "Opacity change emits command for undo/redo",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.4.1"),
                    RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: true,
                    file_path: Some("src/ui/phase0_shell/style_controls.rs"),
                    notes: "Stroke opacity control exists in current shell",
                },
            },
        ]
    }

    /// Style Panel fill controls (3 controls).
    fn style_panel_fill_controls() -> Vec<RequiredControl> {
        vec![
            RequiredControl {
                name: "Fill Color Picker",
                phase: Phase::Phase1,
                surface: ControlSurface::StylePanel,
                user_job: UserJob {
                    description: "Select fill color for shapes",
                },
                states: ControlStates {
                    states: vec!["enabled", "disabled", "focused", "open", "closed"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support keyboard navigation through color picker UI",
                },
                accessibility: AccessibilityRequirements {
                    role: "ColorPicker",
                    label: "Fill Color",
                    states: vec!["focusable", "expanded", "collapsed"],
                    notes: "Must announce current color value and no-fill state",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("SetFillColor"),
                    notes: "Color change emits command for undo/redo",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.4.1"),
                    RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                    RequirementSource::Architecture("14.1"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: true,
                    file_path: Some("src/ui/phase0_shell/style_controls.rs"),
                    notes: "Color picker for fill exists in current shell",
                },
            },
            RequiredControl {
                name: "Fill Opacity Slider",
                phase: Phase::Phase1,
                surface: ControlSurface::StylePanel,
                user_job: UserJob {
                    description: "Adjust fill transparency from 0% to 100%",
                },
                states: ControlStates {
                    states: vec!["enabled", "disabled", "focused"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support arrow keys for value adjustment",
                },
                accessibility: AccessibilityRequirements {
                    role: "Slider",
                    label: "Fill Opacity",
                    states: vec!["focusable"],
                    notes: "Must announce current percentage value",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("SetFillOpacity"),
                    notes: "Opacity change emits command for undo/redo",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.4.1"),
                    RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented separately; may be combined with color picker",
                },
            },
            RequiredControl {
                name: "No Fill Toggle",
                phase: Phase::Phase1,
                surface: ControlSurface::StylePanel,
                user_job: UserJob {
                    description: "Remove fill from shapes",
                },
                states: ControlStates {
                    states: vec!["enabled", "disabled", "focused", "toggled"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support Space or Enter to toggle",
                },
                accessibility: AccessibilityRequirements {
                    role: "ToggleButton",
                    label: "No Fill",
                    states: vec!["focusable", "checked", "unchecked"],
                    notes: "Must announce toggled state",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("ToggleNoFill"),
                    notes: "Toggle emits command for undo/redo",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.4.1"),
                    RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented as explicit toggle",
                },
            },
        ]
    }

    /// Layers Panel controls (5 controls).
    #[expect(
        clippy::too_many_lines,
        reason = "Flat data table for Layers Panel controls; no meaningful sub-structure to extract"
    )]
    fn layers_panel_controls() -> Vec<RequiredControl> {
        vec![
            RequiredControl {
                name: "Layer Row",
                phase: Phase::Phase1,
                surface: ControlSurface::LayersPanel,
                user_job: UserJob {
                    description: "Represent and select a layer in the document hierarchy",
                },
                states: ControlStates {
                    states: vec!["selected", "focused", "dragging", "locked", "hidden"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support arrow keys for navigation, Enter to rename",
                },
                accessibility: AccessibilityRequirements {
                    role: "TreeItem",
                    label: "Layer",
                    states: vec!["focusable", "selected", "expanded", "collapsed"],
                    notes: "Must announce layer name, visibility, and lock state",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("SelectLayer"),
                    notes: "Layer selection routes through action system",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.5.1"),
                    RequirementSource::FeaturePlan("Phase 1: Layers Panel"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented",
                },
            },
            RequiredControl {
                name: "Layer Visibility Toggle",
                phase: Phase::Phase1,
                surface: ControlSurface::LayersPanel,
                user_job: UserJob {
                    description: "Show or hide a layer",
                },
                states: ControlStates {
                    states: vec!["visible", "hidden", "focused"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support Space or Enter to toggle",
                },
                accessibility: AccessibilityRequirements {
                    role: "ToggleButton",
                    label: "Layer Visibility",
                    states: vec!["focusable", "checked", "unchecked"],
                    notes: "Must announce visibility state change",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("ToggleLayerVisibility"),
                    notes: "Visibility toggle must be undoable",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.5.1"),
                    RequirementSource::FeaturePlan("Phase 1: Layers Panel"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented",
                },
            },
            RequiredControl {
                name: "Layer Lock Toggle",
                phase: Phase::Phase1,
                surface: ControlSurface::LayersPanel,
                user_job: UserJob {
                    description: "Lock or unlock a layer to prevent edits",
                },
                states: ControlStates {
                    states: vec!["locked", "unlocked", "focused"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support Space or Enter to toggle",
                },
                accessibility: AccessibilityRequirements {
                    role: "ToggleButton",
                    label: "Layer Lock",
                    states: vec!["focusable", "checked", "unchecked"],
                    notes: "Must announce lock state change",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("ToggleLayerLock"),
                    notes: "Lock toggle must be undoable",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.5.1"),
                    RequirementSource::FeaturePlan("Phase 1: Layers Panel"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented",
                },
            },
            RequiredControl {
                name: "Layer Rename Field",
                phase: Phase::Phase1,
                surface: ControlSurface::LayersPanel,
                user_job: UserJob {
                    description: "Edit layer name inline",
                },
                states: ControlStates {
                    states: vec!["editing", "read-only", "focused"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support Enter to commit, Escape to cancel",
                },
                accessibility: AccessibilityRequirements {
                    role: "TextInput",
                    label: "Layer Name",
                    states: vec!["focusable", "editable"],
                    notes: "Must announce current layer name",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("RenameLayer"),
                    notes: "Rename must be undoable",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.5.1"),
                    RequirementSource::FeaturePlan("Phase 1: Layers Panel"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented",
                },
            },
            RequiredControl {
                name: "Layer Reorder Handle",
                phase: Phase::Phase1,
                surface: ControlSurface::LayersPanel,
                user_job: UserJob {
                    description: "Drag to reorder layers in stacking order",
                },
                states: ControlStates {
                    states: vec!["idle", "dragging", "focused"],
                },
                keyboard: KeyboardRequirements {
                    shortcut: None,
                    keyboard_only_operation: true,
                    notes: "Must support keyboard-only reordering (e.g., Cmd+Up/Down)",
                },
                accessibility: AccessibilityRequirements {
                    role: "Button",
                    label: "Reorder Layer",
                    states: vec!["focusable"],
                    notes: "Must announce position changes during reorder",
                },
                action_linkage: ActionCommandLinkage {
                    requires_action: true,
                    action_name: Some("ReorderLayer"),
                    notes: "Reorder must be undoable",
                },
                sources: vec![
                    RequirementSource::Roadmap("1.5.2"),
                    RequirementSource::FeaturePlan("Phase 1: Layers Panel"),
                ],
                current_evidence: CurrentShellEvidence {
                    exists: false,
                    file_path: None,
                    notes: "Not yet implemented",
                },
            },
        ]
    }

    /// History Panel controls (1 control).
    fn history_panel_controls() -> Vec<RequiredControl> {
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

    /// Character Panel controls (6 controls - Phase 2).
    #[expect(
        clippy::too_many_lines,
        reason = "Flat data table for Character Panel controls; no meaningful sub-structure to extract"
    )]
    fn character_panel_controls() -> Vec<RequiredControl> {
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

    /// Paragraph Panel controls (3 controls - Phase 2).
    fn paragraph_panel_controls() -> Vec<RequiredControl> {
        vec![
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
            },
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
            },
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
            },
        ]
    }

    /// Canvas Text Editor controls (1 control - Phase 2).
    fn canvas_text_editor_controls() -> Vec<RequiredControl> {
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

    /// Returns all controls in the inventory.
    #[must_use]
    pub fn all(&self) -> &[RequiredControl] {
        &self.controls
    }

    /// Returns controls filtered by phase.
    #[must_use]
    pub fn by_phase(&self, phase: Phase) -> Vec<&RequiredControl> {
        self.controls.iter().filter(|c| c.phase == phase).collect()
    }

    /// Returns controls filtered by surface.
    #[must_use]
    pub fn by_surface(&self, surface: ControlSurface) -> Vec<&RequiredControl> {
        self.controls
            .iter()
            .filter(|c| c.surface == surface)
            .collect()
    }

    /// Returns controls that have current shell evidence.
    #[must_use]
    pub fn with_evidence(&self) -> Vec<&RequiredControl> {
        self.controls
            .iter()
            .filter(|c| c.current_evidence.exists)
            .collect()
    }

    /// Returns controls without current shell evidence.
    #[must_use]
    pub fn without_evidence(&self) -> Vec<&RequiredControl> {
        self.controls
            .iter()
            .filter(|c| !c.current_evidence.exists)
            .collect()
    }
}

impl Default for ControlInventory {
    fn default() -> Self {
        Self::new()
    }
}
