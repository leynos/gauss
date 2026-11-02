//! Holds typed literal Make assignments and branch selectors for the source audit.

/// Make assignment operators in the shared vocabulary and their source-reader priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AssignmentOperator {
    /// Assign only when the variable is undefined.
    Conditional,
    /// Expand the value immediately.
    Immediate,
    /// Append the value to the variable.
    Append,
    /// Run a shell command to produce the value.
    Shell,
    /// Expand the value when the variable is referenced.
    Recursive,
}

impl AssignmentOperator {
    /// Shared operator inspection order used by both intentionally distinct readers.
    pub(super) const ALL: [Self; 5] = [
        Self::Conditional,
        Self::Immediate,
        Self::Append,
        Self::Shell,
        Self::Recursive,
    ];

    /// Returns the exact Make spelling for this operator.
    /// For example, `Conditional.spelling()` returns `"?="`.
    pub(super) const fn spelling(self) -> &'static str {
        match self {
            Self::Conditional => "?=",
            Self::Immediate => ":=",
            Self::Append => "+=",
            Self::Shell => "!=",
            Self::Recursive => "=",
        }
    }
}

/// Named source slices returned by either Make assignment reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MakeAssignment<'a> {
    /// Variable name as accepted by the specific reader.
    pub(super) name: &'a str,
    /// Assignment operation represented without stringly typed spelling.
    pub(super) operator: AssignmentOperator,
    /// Trimmed value slice retained from the source.
    pub(super) value: &'a str,
}

/// Closed set of literal selectors accepted from workflow-controlled Make source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) enum MakeSelector {
    /// Operating system value resolved from the workflow runner.
    BuildHostOs,
    /// Explicit test mode supplied by the workflow caller.
    Mode,
}

impl MakeSelector {
    /// Parses only the two supported literal selector names.
    /// For example, `MODE` returns `Ok(Self::Mode)` and `CC` returns an error.
    pub(in super::super) fn parse(name: &str) -> Result<Self, String> {
        match name {
            "BUILD_HOST_OS" => Ok(Self::BuildHostOs),
            "MODE" => Ok(Self::Mode),
            other => Err(format!("tool-dependent or unknown Make selector: {other}")),
        }
    }

    /// Returns the exact source spelling for this selector.
    /// For example, `BuildHostOs.name()` returns `"BUILD_HOST_OS"`.
    pub(in super::super) const fn name(self) -> &'static str {
        match self {
            Self::BuildHostOs => "BUILD_HOST_OS",
            Self::Mode => "MODE",
        }
    }
}

/// A parsed selector and its raw, non-empty comparison literal.
pub(super) struct ParsedCondition<'a> {
    /// Closed selector chosen by the source parser.
    pub(super) selector: MakeSelector,
    /// Comparison value retained for Make to evaluate in its own conditional.
    pub(super) comparison: &'a str,
}
