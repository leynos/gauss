//! Type definitions for widget audit phases, surfaces, controls, and requirements.

use std::fmt;

/// Phase identifier for roadmap requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Phase 1: MVP Core Editing Tools and Foundation
    Phase1,
    /// Phase 2: Text and Advanced Styling
    Phase2,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phase1 => write!(f, "Phase 1"),
            Self::Phase2 => write!(f, "Phase 2"),
        }
    }
}

/// UI surface type where a control appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlSurface {
    /// Tool selection toolbar (left rail or top bar)
    Toolbar,
    /// Properties inspector panel (width, height, rotation, etc.)
    PropertiesPanel,
    /// Stroke and fill styling panel
    StylePanel,
    /// Layers list panel
    LayersPanel,
    /// History/undo panel
    HistoryPanel,
    /// Character formatting panel (Phase 2)
    CharacterPanel,
    /// Paragraph formatting panel (Phase 2)
    ParagraphPanel,
    /// Alignment and distribution controls
    AlignmentPanel,
    /// On-canvas text editing affordance
    CanvasTextEditor,
    /// Popover or contextual control
    Popover,
}

impl fmt::Display for ControlSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toolbar => write!(f, "Toolbar"),
            Self::PropertiesPanel => write!(f, "Properties Panel"),
            Self::StylePanel => write!(f, "Style Panel"),
            Self::LayersPanel => write!(f, "Layers Panel"),
            Self::HistoryPanel => write!(f, "History Panel"),
            Self::CharacterPanel => write!(f, "Character Panel"),
            Self::ParagraphPanel => write!(f, "Paragraph Panel"),
            Self::AlignmentPanel => write!(f, "Alignment Panel"),
            Self::CanvasTextEditor => write!(f, "Canvas Text Editor"),
            Self::Popover => write!(f, "Popover"),
        }
    }
}

/// Requirement source document reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementSource {
    /// Roadmap item (e.g., "1.2.3").
    Roadmap(&'static str),
    /// Feature plan reference.
    FeaturePlan(&'static str),
    /// Architecture decision reference.
    Architecture(&'static str),
}

impl fmt::Display for RequirementSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Roadmap(id) => write!(f, "Roadmap {id}"),
            Self::FeaturePlan(name) => write!(f, "Feature Plan: {name}"),
            Self::Architecture(section) => write!(f, "Architecture §{section}"),
        }
    }
}

/// User-facing job or capability the control supports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserJob {
    /// Brief description of what the user accomplishes with this control.
    pub description: &'static str,
}

/// Required states a control must expose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlStates {
    /// List of states (e.g., "selected", "disabled", "focused", "active").
    pub states: Vec<&'static str>,
}

/// Keyboard interaction requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardRequirements {
    /// Keyboard shortcut if applicable (e.g., "V" for Selection tool).
    pub shortcut: Option<&'static str>,
    /// Whether control must support keyboard-only operation.
    pub keyboard_only_operation: bool,
    /// Additional notes about keyboard requirements.
    pub notes: &'static str,
}

/// Accessibility requirements per AccessKit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessibilityRequirements {
    /// AccessKit role (e.g., "Button", "`TextInput`", "Slider").
    pub role: &'static str,
    /// Accessible label.
    pub label: &'static str,
    /// Required accessibility states.
    pub states: Vec<&'static str>,
    /// Additional accessibility notes.
    pub notes: &'static str,
}

/// Action/Command pipeline linkage requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionCommandLinkage {
    /// Whether this control requires integration with the Action system.
    pub requires_action: bool,
    /// Action name if applicable.
    pub action_name: Option<&'static str>,
    /// Notes about action linkage.
    pub notes: &'static str,
}

/// Evidence of current shell implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentShellEvidence {
    /// Whether evidence exists in the current Phase 0 shell.
    pub exists: bool,
    /// File path where evidence can be found, if it exists.
    pub file_path: Option<&'static str>,
    /// Notes about the current implementation or absence thereof.
    pub notes: &'static str,
}

/// A required UI control with full specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredControl {
    /// Control name (e.g., "Selection Tool", "X Position Field").
    pub name: &'static str,
    /// Which roadmap phase requires this control.
    pub phase: Phase,
    /// UI surface where this control appears.
    pub surface: ControlSurface,
    /// User job this control supports.
    pub user_job: UserJob,
    /// Required states this control must expose.
    pub states: ControlStates,
    /// Keyboard interaction requirements.
    pub keyboard: KeyboardRequirements,
    /// Accessibility requirements.
    pub accessibility: AccessibilityRequirements,
    /// Action/Command pipeline linkage.
    pub action_linkage: ActionCommandLinkage,
    /// Source documents justifying this requirement.
    pub sources: Vec<RequirementSource>,
    /// Current shell implementation evidence.
    pub current_evidence: CurrentShellEvidence,
}

impl RequiredControl {
    /// Returns the control's name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the phase requiring this control.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// Returns the surface this control appears on.
    #[must_use]
    pub const fn surface(&self) -> ControlSurface {
        self.surface
    }

    /// Checks if current shell evidence exists for this control.
    #[must_use]
    pub const fn has_current_evidence(&self) -> bool {
        self.current_evidence.exists
    }
}
