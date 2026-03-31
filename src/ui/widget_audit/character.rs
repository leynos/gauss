//! Character panel control definitions.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

#[derive(Clone, Copy)]
struct CharacterControlSpec {
    name: &'static str,
    description: &'static str,
    shortcut: Option<&'static str>,
    keyboard_notes: &'static str,
    control_states: &'static [&'static str],
    a11y_role: &'static str,
    a11y_label: &'static str,
    a11y_states: &'static [&'static str],
    a11y_notes: &'static str,
    action_name: &'static str,
    action_notes: &'static str,
    evidence_notes: &'static str,
}

fn make_character_control(spec: CharacterControlSpec) -> RequiredControl {
    RequiredControl {
        name: spec.name,
        phase: Phase::Phase2,
        surface: ControlSurface::CharacterPanel,
        user_job: UserJob {
            description: spec.description,
        },
        states: ControlStates {
            states: spec.control_states.to_vec(),
        },
        keyboard: KeyboardRequirements {
            shortcut: spec.shortcut,
            keyboard_only_operation: true,
            notes: spec.keyboard_notes,
        },
        accessibility: AccessibilityRequirements {
            role: spec.a11y_role,
            label: spec.a11y_label,
            states: spec.a11y_states.to_vec(),
            notes: spec.a11y_notes,
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some(spec.action_name),
            notes: spec.action_notes,
        },
        sources: vec![
            RequirementSource::Roadmap("2.2"),
            RequirementSource::FeaturePlan("Phase 2: Text and Advanced Styling"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: spec.evidence_notes,
        },
    }
}

/// Returns the character panel control inventory.
///
/// Combines font property controls (family selector, size field) with text
/// style controls (bold, italic, alignment, colour picker).
pub(super) fn controls() -> Vec<RequiredControl> {
    let mut v = font_property_controls();
    v.extend(text_style_controls());
    v
}

fn font_property_controls() -> Vec<RequiredControl> {
    vec![
        make_character_control(CharacterControlSpec {
            name: "Font Family Selector",
            description: "Select font family for text",
            shortcut: None,
            keyboard_notes: "Must support keyboard navigation and search through font list",
            control_states: &["enabled", "disabled", "focused", "open", "closed"],
            a11y_role: "ComboBox",
            a11y_label: "Font Family",
            a11y_states: &["focusable", "expanded", "collapsed"],
            a11y_notes: "Must announce current font family and selection changes",
            action_name: "SetFontFamily",
            action_notes: "Font change must be undoable",
            evidence_notes: "Phase 2 feature",
        }),
        make_character_control(CharacterControlSpec {
            name: "Font Size Field",
            description: "Set font size in points",
            shortcut: None,
            keyboard_notes: "Must support keyboard input, arrow keys for nudge",
            control_states: &["enabled", "disabled", "focused"],
            a11y_role: "TextInput",
            a11y_label: "Font Size",
            a11y_states: &["focusable", "editable"],
            a11y_notes: "Must announce current size in points",
            action_name: "SetFontSize",
            action_notes: "Size change must be undoable",
            evidence_notes: "Phase 2 feature",
        }),
    ]
}

fn text_style_controls() -> Vec<RequiredControl> {
    vec![
        make_character_control(CharacterControlSpec {
            name: "Bold Toggle",
            description: "Apply or remove bold formatting",
            shortcut: Some("Cmd+B / Ctrl+B"),
            keyboard_notes: "Standard bold shortcut must work",
            control_states: &["enabled", "disabled", "focused", "active"],
            a11y_role: "ToggleButton",
            a11y_label: "Bold",
            a11y_states: &["focusable", "checked", "unchecked"],
            a11y_notes: "Must announce formatting state",
            action_name: "ToggleBold",
            action_notes: "Formatting change must be undoable",
            evidence_notes: "Phase 2 feature",
        }),
        make_character_control(CharacterControlSpec {
            name: "Italic Toggle",
            description: "Apply or remove italic formatting",
            shortcut: Some("Cmd+I / Ctrl+I"),
            keyboard_notes: "Standard italic shortcut must work",
            control_states: &["enabled", "disabled", "focused", "active"],
            a11y_role: "ToggleButton",
            a11y_label: "Italic",
            a11y_states: &["focusable", "checked", "unchecked"],
            a11y_notes: "Must announce formatting state",
            action_name: "ToggleItalic",
            action_notes: "Formatting change must be undoable",
            evidence_notes: "Phase 2 feature",
        }),
        make_character_control(CharacterControlSpec {
            name: "Text Alignment Buttons",
            description: "Set text alignment (left, center, right, justify)",
            shortcut: None,
            keyboard_notes: "Must support keyboard navigation between alignment options",
            control_states: &["enabled", "disabled", "focused", "selected"],
            a11y_role: "RadioGroup",
            a11y_label: "Text Alignment",
            a11y_states: &["focusable"],
            a11y_notes: "Must announce current alignment selection",
            action_name: "SetTextAlignment",
            action_notes: "Alignment change must be undoable",
            evidence_notes: "Phase 2 feature",
        }),
        make_character_control(CharacterControlSpec {
            name: "Text Color Picker",
            description: "Set color for text characters",
            shortcut: None,
            keyboard_notes: "Must support keyboard navigation through color picker",
            control_states: &["enabled", "disabled", "focused", "open", "closed"],
            a11y_role: "ColorPicker",
            a11y_label: "Text Color",
            a11y_states: &["focusable", "expanded", "collapsed"],
            a11y_notes: "Must announce current color value; reuses fill/stroke color infrastructure per roadmap 2.2",
            action_name: "SetTextColor",
            action_notes: "Color change must be undoable",
            evidence_notes: "Phase 2 feature; will reuse color picker infrastructure",
        }),
    ]
}
