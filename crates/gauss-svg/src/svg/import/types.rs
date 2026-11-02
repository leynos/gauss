//! Domain-specific string wrappers for SVG import parsing.

/// Define a borrowed string newtype for a specific SVG parsing role.
///
/// Each generated type is a `Copy` wrapper with `new`, `as_str`, `From<&str>`,
/// and `AsRef<str>` so role confusion between unrelated strings is prevented.
macro_rules! string_wrapper {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(super) struct $name<'a>(&'a str);

        impl<'a> $name<'a> {
            pub(super) const fn new(inner: &'a str) -> Self { Self(inner) }

            pub(super) const fn as_str(&self) -> &'a str { self.0 }
        }

        impl<'a> From<&'a str> for $name<'a> {
            fn from(value: &'a str) -> Self { Self(value) }
        }

        impl AsRef<str> for $name<'_> {
            fn as_ref(&self) -> &str { self.as_str() }
        }
    };
}

string_wrapper!(TagName);
string_wrapper!(AttributeName);
string_wrapper!(SvgContent);
