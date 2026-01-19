//! Tool and edge mode definitions for the Gauss editor.
//!
//! This module defines the primary interaction modes for the editor. Tool mode
//! determines which tool is active (e.g., pen for drawing, selection for
//! manipulation). Edge mode determines how new path segments are created.
//!
//! These types are GPUI-independent for testability and scripting.

/// The active tool in the editor.
///
/// Tool mode determines how user input is interpreted. In Draw mode, clicks
/// place anchors to create paths. In Manipulate mode, clicks select and move
/// existing shapes.
///
/// # Examples
///
/// ```rust
/// use gauss::model::ToolMode;
///
/// let mode = ToolMode::Draw;
/// assert_eq!(mode.label(), "Draw");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolMode {
    /// Draw mode: clicks place anchors to create new paths.
    #[default]
    Draw,
    /// Manipulate mode: clicks select and move existing shapes.
    Manipulate,
}

impl ToolMode {
    /// Return a human-readable label for this tool mode.
    ///
    /// Used for status line display and accessibility labels.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Manipulate => "Manipulate",
        }
    }
}

/// The edge mode for path segment creation.
///
/// Edge mode determines how new segments are connected when drawing paths.
/// Line mode creates straight segments. Bezier (auto) mode creates smooth
/// curves with automatically calculated handles.
///
/// # Examples
///
/// ```rust
/// use gauss::model::EdgeMode;
///
/// let mode = EdgeMode::Line;
/// assert_eq!(mode.label(), "Line");
/// assert_eq!(mode.toggle(), EdgeMode::BezierAuto);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeMode {
    /// Straight line segments between anchors.
    #[default]
    Line,
    /// Smooth curves with Catmull-Rom interpolated handles.
    BezierAuto,
}

impl EdgeMode {
    /// Return a human-readable label for this edge mode.
    ///
    /// Used for status line display and accessibility labels.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::BezierAuto => "Bezier (auto)",
        }
    }

    /// Return the opposite edge mode.
    ///
    /// Used for Tab key toggle behaviour.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Line => Self::BezierAuto,
            Self::BezierAuto => Self::Line,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for tool and edge mode helpers.

    use super::*;
    use rstest::rstest;

    #[rstest]
    fn tool_mode_has_label() {
        assert_eq!(ToolMode::Draw.label(), "Draw");
        assert_eq!(ToolMode::Manipulate.label(), "Manipulate");
    }

    #[rstest]
    fn tool_mode_default_is_draw() {
        assert_eq!(ToolMode::default(), ToolMode::Draw);
    }

    #[rstest]
    fn edge_mode_has_label() {
        assert_eq!(EdgeMode::Line.label(), "Line");
        assert_eq!(EdgeMode::BezierAuto.label(), "Bezier (auto)");
    }

    #[rstest]
    fn edge_mode_default_is_line() {
        assert_eq!(EdgeMode::default(), EdgeMode::Line);
    }

    #[rstest]
    fn edge_mode_toggle_switches() {
        assert_eq!(EdgeMode::Line.toggle(), EdgeMode::BezierAuto);
        assert_eq!(EdgeMode::BezierAuto.toggle(), EdgeMode::Line);
    }

    #[rstest]
    fn tool_mode_is_copy() {
        let mode = ToolMode::Draw;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[rstest]
    fn edge_mode_is_copy() {
        let mode = EdgeMode::Line;
        let copied = mode;
        assert_eq!(mode, copied);
    }
}
