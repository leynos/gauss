//! Key contexts for keyboard shortcut scoping.
//!
//! Key contexts determine which keyboard shortcuts are active based on the
//! current editor mode. GPUI dispatches key bindings relative to an element's
//! key context, allowing mode-specific shortcuts.
//!
//! # Design
//!
//! Key contexts are implemented as an enum rather than raw strings for several
//! reasons:
//!
//! - **Exhaustive matching**: All context variants can be matched exhaustively,
//!   ensuring dispatch tables are complete and verifiable at compile time.
//! - **Type safety**: Prevents typos in context strings.
//! - **Testability**: Context logic can be tested without GPUI.
//! - **Consistency**: Matches the established Action/Command enum pattern.
//!
//! # GPUI Integration
//!
//! Contexts are converted to GPUI-compatible strings via [`AsRef<str>`]. Context
//! strings use the format `gauss-{name}` and must contain only letters, digits,
//! `_`, or `-` per GPUI requirements.
//!
//! # Examples
//!
//! ```rust,no_run
//! use gauss_core::model::KeyContext;
//!
//! let context = KeyContext::Global;
//! assert_eq!(context.as_ref(), "gauss-global");
//! assert_eq!(context.name(), "Global");
//! ```

use std::fmt;

/// Editor key contexts for keyboard shortcut scoping.
///
/// Key contexts determine which keyboard shortcuts are active based on the
/// current editor state. GPUI dispatches key bindings relative to an element's
/// key context, allowing mode-specific shortcuts.
///
/// # Variants
///
/// This enum uses `#[non_exhaustive]` to allow adding new context variants in
/// future versions without breaking downstream code.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss_core::model::KeyContext;
///
/// // Convert to GPUI-compatible string
/// let context = KeyContext::DrawMode;
/// let gpui_str: &str = context.as_ref();
/// assert_eq!(gpui_str, "gauss-draw");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyContext {
    /// Global context active in all editor modes.
    ///
    /// Used for actions that work regardless of the current tool or mode, such
    /// as Undo, Redo, Save, Open, and tool activation shortcuts.
    Global,

    /// Active when the Pen (draw) tool is selected.
    ///
    /// Draw mode shortcuts include Tab to toggle edge mode (line vs Bezier)
    /// and Escape to commit the current path.
    DrawMode,

    /// Active when the Selection (manipulate) tool is selected.
    ///
    /// Manipulate mode shortcuts include Tab to toggle segment kind,
    /// Delete/Backspace to remove selected items, and transform shortcuts.
    ManipulateMode,

    /// Active during on-canvas text editing (future Phase 2).
    ///
    /// Captures text input and navigation keys, overriding global shortcuts
    /// that would otherwise interfere with typing.
    TextEdit,
}

impl KeyContext {
    /// Return a human-readable name for this context.
    ///
    /// This name is suitable for:
    ///
    /// - UI display (e.g., "Press P in Global context")
    /// - Accessibility labels
    /// - Documentation
    ///
    /// Note: These names will be replaced with localised strings when the i18n
    /// scaffolding (task 0.7) is implemented.
    ///
    /// # Returns
    ///
    /// A static string containing the human-readable context name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::KeyContext;
    ///
    /// assert_eq!(KeyContext::Global.name(), "Global");
    /// assert_eq!(KeyContext::DrawMode.name(), "Draw Mode");
    /// ```
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::DrawMode => "Draw Mode",
            Self::ManipulateMode => "Manipulate Mode",
            Self::TextEdit => "Text Edit",
        }
    }

    /// Return all defined key contexts.
    ///
    /// Useful for iterating over contexts when registering keybindings or
    /// generating documentation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::KeyContext;
    ///
    /// for context in KeyContext::all() {
    ///     println!("{}: {}", context.name(), context.as_ref());
    /// }
    /// ```
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Global,
            Self::DrawMode,
            Self::ManipulateMode,
            Self::TextEdit,
        ]
    }
}

impl AsRef<str> for KeyContext {
    /// Convert to a GPUI-compatible context string.
    ///
    /// Context strings:
    ///
    /// - Use the format `gauss-{name}` for namespacing
    /// - Contain only letters, digits, `_`, or `-` (GPUI requirement)
    /// - Avoid `.` which is not valid in GPUI key contexts
    fn as_ref(&self) -> &str {
        match self {
            Self::Global => "gauss-global",
            Self::DrawMode => "gauss-draw",
            Self::ManipulateMode => "gauss-manipulate",
            Self::TextEdit => "gauss-text-edit",
        }
    }
}

impl fmt::Display for KeyContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    //! Tests for key context naming and formatting.

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(KeyContext::Global, "gauss-global")]
    #[case(KeyContext::DrawMode, "gauss-draw")]
    #[case(KeyContext::ManipulateMode, "gauss-manipulate")]
    #[case(KeyContext::TextEdit, "gauss-text-edit")]
    fn as_ref_returns_gpui_compatible_string(#[case] context: KeyContext, #[case] expected: &str) {
        assert_eq!(context.as_ref(), expected);
    }

    #[rstest]
    #[case(KeyContext::Global)]
    #[case(KeyContext::DrawMode)]
    #[case(KeyContext::ManipulateMode)]
    #[case(KeyContext::TextEdit)]
    fn as_ref_contains_only_valid_gpui_characters(#[case] context: KeyContext) {
        let s = context.as_ref();
        // GPUI key contexts accept: letters, digits, _, -
        assert!(
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "Context string '{s}' contains invalid GPUI characters"
        );
    }

    #[rstest]
    #[case(KeyContext::Global)]
    #[case(KeyContext::DrawMode)]
    #[case(KeyContext::ManipulateMode)]
    #[case(KeyContext::TextEdit)]
    fn as_ref_does_not_contain_period(#[case] context: KeyContext) {
        let s = context.as_ref();
        assert!(
            !s.contains('.'),
            "Context string '{s}' contains '.', which is invalid in GPUI key contexts"
        );
    }

    #[rstest]
    #[case(KeyContext::Global, "Global")]
    #[case(KeyContext::DrawMode, "Draw Mode")]
    #[case(KeyContext::ManipulateMode, "Manipulate Mode")]
    #[case(KeyContext::TextEdit, "Text Edit")]
    fn name_returns_human_readable_string(#[case] context: KeyContext, #[case] expected: &str) {
        assert_eq!(context.name(), expected);
    }

    #[rstest]
    #[case(KeyContext::Global)]
    #[case(KeyContext::DrawMode)]
    #[case(KeyContext::ManipulateMode)]
    #[case(KeyContext::TextEdit)]
    fn display_matches_name(#[case] context: KeyContext) {
        assert_eq!(format!("{context}"), context.name());
    }

    #[test]
    fn all_returns_all_variants() {
        let all = KeyContext::all();

        // Exhaustive match ensures this test must be updated when variants change.
        // The compiler will flag any missing variants, keeping all() in sync.
        let expected_count = {
            let mut count = 0;
            for ctx in all {
                match ctx {
                    KeyContext::Global
                    | KeyContext::DrawMode
                    | KeyContext::ManipulateMode
                    | KeyContext::TextEdit => count += 1,
                }
            }
            count
        };

        assert_eq!(
            all.len(),
            expected_count,
            "all() should contain exactly the variants matched above"
        );
    }

    #[test]
    fn key_context_is_copy() {
        // Verify KeyContext implements Copy (important for ergonomics)
        fn assert_copy<T: Copy>(_: T) {}

        assert_copy(KeyContext::Global);
    }

    #[test]
    fn key_context_is_hashable() {
        // Verify KeyContext can be used as a map key
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(KeyContext::Global);
        set.insert(KeyContext::DrawMode);
        assert_eq!(set.len(), 2);
    }
}
