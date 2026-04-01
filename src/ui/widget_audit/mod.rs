//! Widget capability audit for Phase 1-2 controls.
//!
//! This module defines a typed inventory of all UI controls required by the
//! Phase 1 and Phase 2 roadmap. It serves as the canonical source of truth for
//! control requirements before implementation begins.
//!
//! Each control entry records:
//! - Which phase requires it
//! - Which surface it belongs to (toolbar, panel, etc.)
//! - What user job it supports
//! - What states it must expose
//! - Keyboard and accessibility requirements
//! - Source documents that justify its inclusion
//! - Whether current shell evidence exists
//!
//! This module is public to enable integration testing and external validation
//! of the control inventory against roadmap requirements.

mod action;
mod alignment;
mod canvas;
mod character;
mod history;
mod layers;
mod paragraph;
mod properties;
mod style;
mod toolbar;
mod types;

pub use action::AuditAction;
pub use types::{
    AccessibilityRequirements, ActionCommandLinkage, ControlStates, ControlSurface,
    CurrentShellEvidence, KeyboardRequirements, Phase, RequiredControl, RequirementSource, UserJob,
};

/// Complete inventory of required controls for Phase 1-2.
pub struct ControlInventory {
    controls: Vec<RequiredControl>,
}

impl ControlInventory {
    /// Creates a new inventory with the complete Phase 1-2 control set.
    #[must_use]
    pub fn new() -> Self {
        let mut controls = Vec::new();
        controls.extend(toolbar::controls());
        controls.extend(properties::controls());
        controls.extend(alignment::controls());
        controls.extend(style::controls());
        controls.extend(layers::controls());
        controls.extend(history::controls());
        controls.extend(character::controls());
        controls.extend(paragraph::controls());
        controls.extend(canvas::controls());
        Self { controls }
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
        self.filter_by_evidence(true)
    }

    /// Returns controls without current shell evidence.
    #[must_use]
    pub fn without_evidence(&self) -> Vec<&RequiredControl> {
        self.filter_by_evidence(false)
    }

    /// Filter controls by evidence existence.
    fn filter_by_evidence(&self, exists: bool) -> Vec<&RequiredControl> {
        self.controls
            .iter()
            .filter(|c| c.current_evidence.exists == exists)
            .collect()
    }
}

impl Default for ControlInventory {
    fn default() -> Self {
        Self::new()
    }
}
