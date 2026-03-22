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
//! This is an internal planning artefact, not a public product API.

pub mod types;
pub mod control;
pub mod inventory;

// Flatten the public API so existing use paths continue to work.
pub use types::{Phase, ControlSurface, RequirementSource};
pub use control::{
    UserJob, ControlStates, KeyboardRequirements, AccessibilityRequirements,
    ActionCommandLinkage, CurrentShellEvidence, RequiredControl,
};
pub use inventory::ControlInventory;
