//! Style panel control definitions (stroke and fill).

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

/// Style panel controls (stroke and fill).
pub(super) fn controls() -> Vec<RequiredControl> {
    let mut v = stroke_controls();
    v.extend(fill_controls());
    v
}

fn stroke_controls() -> Vec<RequiredControl> {
    vec![stroke_color_picker(), stroke_width_field(), stroke_opacity_slider()]
}

fn stroke_color_picker() -> RequiredControl {
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
    }
}

fn stroke_width_field() -> RequiredControl {
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
    }
}

fn stroke_opacity_slider() -> RequiredControl {
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
    }
}

fn fill_controls() -> Vec<RequiredControl> {
    vec![fill_color_picker(), fill_opacity_slider(), no_fill_toggle()]
}

fn fill_color_picker() -> RequiredControl {
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
    }
}

fn fill_opacity_slider() -> RequiredControl {
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
    }
}

fn no_fill_toggle() -> RequiredControl {
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
    }
}
