//! Style panel control definitions (stroke and fill).

use super::{
    AccessibilityRequirements,
    ActionCommandLinkage,
    AuditAction,
    ControlStates,
    ControlSurface,
    CurrentShellEvidence,
    KeyboardRequirements,
    Phase,
    RequiredControl,
    RequirementSource,
    UserJob,
};

/// Specification for a style panel control.
#[derive(Clone, Copy)]
struct StyleControlSpec {
    /// Display name of the control.
    name: &'static str,
    /// User job the control supports.
    description: &'static str,
    /// Interaction states the control must expose.
    states: &'static [&'static str],
    /// Accessibility role, such as `ColorPicker` or `Slider`.
    control_type: &'static str,
    /// Accessible label announced to assistive technology.
    label: &'static str,
    /// Accessibility states the control must report.
    a11y_states: &'static [&'static str],
    /// Notes on how state changes must be announced.
    a11y_notes: &'static str,
    /// Command emitted when the control changes.
    action_name: AuditAction,
    /// Undo and routing notes for the emitted command.
    action_notes: &'static str,
    /// Requirement sources justifying the control's inclusion.
    sources: &'static [RequirementSource],
    /// Whether the current shell already provides this control.
    evidence_exists: bool,
    /// Path to the shell implementation; required when `evidence_exists`.
    evidence_path: Option<&'static str>,
    /// Current evidence status for this control.
    evidence_notes: &'static str,
}

/// Creates a `RequiredControl` from a `StyleControlSpec`.
///
/// # Panics
///
/// Panics if `evidence_exists` is true but `evidence_path` is None,
/// as this violates the evidence invariant.
fn make_style_control(spec: StyleControlSpec) -> RequiredControl {
    assert!(
        !(spec.evidence_exists && spec.evidence_path.is_none()),
        "Evidence path must be provided when evidence_exists is true"
    );
    RequiredControl {
        name: spec.name,
        phase: Phase::Phase1,
        surface: ControlSurface::StylePanel,
        user_job: UserJob {
            description: spec.description,
        },
        states: ControlStates {
            states: spec.states.to_vec(),
        },
        keyboard: KeyboardRequirements {
            shortcut: None,
            keyboard_only_operation: true,
            notes: "Must support keyboard navigation",
        },
        accessibility: AccessibilityRequirements {
            role: spec.control_type,
            label: spec.label,
            states: spec.a11y_states.to_vec(),
            notes: spec.a11y_notes,
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some(spec.action_name.identifier()),
            notes: spec.action_notes,
        },
        sources: spec.sources.to_vec(),
        current_evidence: CurrentShellEvidence {
            exists: spec.evidence_exists,
            file_path: spec.evidence_path,
            notes: spec.evidence_notes,
        },
    }
}

/// Returns the style panel control inventory (stroke and fill).
pub(super) fn controls() -> Vec<RequiredControl> {
    let mut v = stroke_controls();
    v.extend(fill_controls());
    v
}

/// Returns the stroke colour, width, and opacity controls.
fn stroke_controls() -> Vec<RequiredControl> {
    vec![
        make_style_control(StyleControlSpec {
            name: "Stroke Colour Picker",
            description: "Select stroke colour for shapes",
            states: &["enabled", "disabled", "focused", "open", "closed"],
            control_type: "ColorPicker",
            label: "Stroke Colour",
            a11y_states: &["focusable", "expanded", "collapsed"],
            a11y_notes: "Must announce current colour value",
            action_name: AuditAction::SetStrokeColor,
            action_notes: "Colour change emits command for undo/redo",
            sources: &[
                RequirementSource::Roadmap("1.4.1"),
                RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                RequirementSource::Architecture("14.1"),
            ],
            evidence_exists: true,
            evidence_path: Some("src/ui/phase0_shell/style_controls.rs"),
            evidence_notes: "Colour picker for stroke exists in current shell",
        }),
        make_style_control(StyleControlSpec {
            name: "Stroke Width Field",
            description: "Set stroke width in pixels or points",
            states: &["enabled", "disabled", "focused", "read-only"],
            control_type: "TextInput",
            label: "Stroke Width",
            a11y_states: &["focusable", "editable", "read-only"],
            a11y_notes: "Must announce current value and units",
            action_name: AuditAction::SetStrokeWidth,
            action_notes: "Width change emits command for undo/redo",
            sources: &[
                RequirementSource::Roadmap("1.4.1"),
                RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
            ],
            evidence_exists: true,
            evidence_path: Some("src/ui/phase0_shell/style_controls.rs"),
            evidence_notes: "Stroke width control exists in current shell",
        }),
        make_style_control(StyleControlSpec {
            name: "Stroke Opacity Slider",
            description: "Adjust stroke transparency from 0% to 100%",
            states: &["enabled", "disabled", "focused"],
            control_type: "Slider",
            label: "Stroke Opacity",
            a11y_states: &["focusable"],
            a11y_notes: "Must announce current percentage value",
            action_name: AuditAction::SetStrokeOpacity,
            action_notes: "Opacity change emits command for undo/redo",
            sources: &[
                RequirementSource::Roadmap("1.4.1"),
                RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
            ],
            evidence_exists: true,
            evidence_path: Some("src/ui/phase0_shell/style_controls.rs"),
            evidence_notes: "Stroke opacity control exists in current shell",
        }),
    ]
}

/// Returns the fill colour, fill opacity, and no-fill controls.
fn fill_controls() -> Vec<RequiredControl> {
    vec![
        make_style_control(StyleControlSpec {
            name: "Fill Colour Picker",
            description: "Select fill colour for shapes",
            states: &["enabled", "disabled", "focused", "open", "closed"],
            control_type: "ColorPicker",
            label: "Fill Colour",
            a11y_states: &["focusable", "expanded", "collapsed"],
            a11y_notes: "Must announce current colour value and no-fill state",
            action_name: AuditAction::SetFillColor,
            action_notes: "Colour change emits command for undo/redo",
            sources: &[
                RequirementSource::Roadmap("1.4.1"),
                RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
                RequirementSource::Architecture("14.1"),
            ],
            evidence_exists: true,
            evidence_path: Some("src/ui/phase0_shell/style_controls.rs"),
            evidence_notes: "Colour picker for fill exists in current shell",
        }),
        make_style_control(StyleControlSpec {
            name: "Fill Opacity Slider",
            description: "Adjust fill transparency from 0% to 100%",
            states: &["enabled", "disabled", "focused"],
            control_type: "Slider",
            label: "Fill Opacity",
            a11y_states: &["focusable"],
            a11y_notes: "Must announce current percentage value",
            action_name: AuditAction::SetFillOpacity,
            action_notes: "Opacity change emits command for undo/redo",
            sources: &[
                RequirementSource::Roadmap("1.4.1"),
                RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
            ],
            evidence_exists: false,
            evidence_path: None,
            evidence_notes: "Not yet implemented separately; may be combined with colour picker",
        }),
        make_style_control(StyleControlSpec {
            name: "No Fill Toggle",
            description: "Remove fill from shapes",
            states: &["enabled", "disabled", "focused", "toggled"],
            control_type: "ToggleButton",
            label: "No Fill",
            a11y_states: &["focusable", "checked", "unchecked"],
            a11y_notes: "Must announce toggled state",
            action_name: AuditAction::ToggleNoFill,
            action_notes: "Toggle emits command for undo/redo",
            sources: &[
                RequirementSource::Roadmap("1.4.1"),
                RequirementSource::FeaturePlan("Phase 1: Styling Tools"),
            ],
            evidence_exists: false,
            evidence_path: None,
            evidence_notes: "Not yet implemented as explicit toggle",
        }),
    ]
}
