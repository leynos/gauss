//! Proc-macros for test fixtures that suppress lints triggered by macro expansion.
//!
//! `rstest` fixtures written as a single expression are kept on one line by
//! `fn_single_line` formatting, and the resulting fixture expansion wraps that
//! expression in braces the compiler then reports as `unused_braces`. Under the
//! workspace's `-D warnings` policy the false positive fails the build even
//! though the source is correctly formatted.
//!
//! This crate provides the `allow_fixture_expansion_lints` attribute, which
//! scopes an `unused_braces` allowance to the expansion of the annotated item.
//! Keeping the allowance scoped matters: a crate-level or file-level `allow`
//! would also hide a genuine redundant brace in handwritten test code.
//!
//! The crate is workspace-private (`publish = false`) and is consumed only as a
//! dev-dependency by test targets that annotate compacted fixtures.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_macro_input};

/// Allows `unused_braces` lint for fixture functions.
///
/// This attribute is used on rstest fixture functions that expand to single-expression
/// bodies. When combined with `fn_single_line = true` in rustfmt.toml, the generated
/// code triggers the `unused_braces` lint. This attribute suppresses that lint
/// specifically for fixture expansions.
///
/// Write it immediately above `#[fixture]` so the suppression wraps the whole
/// expansion rather than only the original item:
///
/// ```ignore
/// #[allow_fixture_expansion_lints]
/// #[fixture]
/// fn world() -> World { World::default() }
/// ```
///
/// The attribute keeps such a fixture compiling under `-D warnings`; without it,
/// Clippy rejects the expansion with `unused_braces`. A fixture whose body rustfmt
/// keeps on several lines needs no attribute, because only the compacted
/// single-expression form triggers the lint.
///
/// The example is `ignore` rather than compiled: this crate is a proc-macro helper
/// with no rstest dependency, so the annotated fixture belongs to the caller's test
/// target.
#[proc_macro_attribute]
pub fn allow_fixture_expansion_lints(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_item = parse_macro_input!(item as Item);

    quote! {
        #[allow(
            unused_braces,
            reason = "fixture macro expansion triggers unused-braces on expression bodies"
        )]
        #[cfg_attr(
            clippy,
            expect(
                clippy::allow_attributes,
                reason = "needed to allow unused_braces for fixture macro expansion"
            )
        )]
        #parsed_item
    }
    .into()
}
