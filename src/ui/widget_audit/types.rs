//! Type definitions for widget audit phases, surfaces, and requirements.

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
