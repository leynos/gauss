//! Layers panel control definitions.
//!
//! This module defines the layer panel controls required for Phase 1, including:
//! - **Layer Row** — Represents and selects a layer in the document hierarchy
//! - **Layer Visibility Toggle** — Shows or hides a layer
//! - **Layer Lock Toggle** — Locks or unlocks a layer to prevent edits
//! - **Layer Rename Field** — Edits layer names inline
//! - **Layer Reorder Handle** — Drags to reorder layers in stacking order
//!
//! These controls are used by the widget capability audit to verify that
//! the Phase 0 shell implements the required layer panel functionality.

use super::types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        layer_row(),
        layer_visibility_toggle(),
        layer_lock_toggle(),
        layer_rename_field(),
        layer_reorder_handle(),
    ]
}

fn layer_row() -> RequiredControl {
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
    }
}

fn layer_visibility_toggle() -> RequiredControl {
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
    }
}

fn layer_lock_toggle() -> RequiredControl {
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
    }
}

fn layer_rename_field() -> RequiredControl {
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
    }
}

fn layer_reorder_handle() -> RequiredControl {
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
    }
}
